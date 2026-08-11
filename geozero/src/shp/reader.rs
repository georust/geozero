use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::iter::FusedIterator;
use std::path::Path;

pub use dbase::{FieldInfo, FieldType};

use crate::shp::shp_reader::{RecordHeader, read_shape};
use crate::shp::shx_reader::{ShapeIndex, read_index_file};
use crate::shp::{Error, header};
use crate::{FeatureProcessor, FeatureProperties, GeomProcessor};

/// Struct that handle iteration over the shapes of a .shp file
pub struct ShapeIterator<'a, P: GeomProcessor, T: Read> {
    processor: &'a mut P,
    source: T,
    current_pos: usize,
    file_length: usize,
    encountered_error: bool,
    /// Shape count from the .shx index, if one was loaded
    remaining: Option<usize>,
}

impl<P: GeomProcessor, T: Read> ShapeIterator<'_, P, T> {
    /// Read one record and advance `current_pos` past it.
    fn read_record(&mut self) -> Result<(), Error> {
        let hdr = read_shape(self.processor, &mut self.source)?;
        // `record_size` is in 16-bit words and comes straight off the record
        // header, so it can be negative or large enough to overflow the doubling.
        let record_size = usize::try_from(hdr.record_size)
            .ok()
            .and_then(|size| size.checked_mul(2))
            .ok_or(Error::InvalidShapeRecordSize)?;
        self.current_pos = self
            .current_pos
            .saturating_add(RecordHeader::SIZE)
            .saturating_add(record_size);
        self.remaining = self.remaining.map(|n| n.saturating_sub(1));
        Ok(())
    }
}

impl<'a, P: GeomProcessor, T: Read + 'a> Iterator for ShapeIterator<'a, P, T> {
    type Item = Result<(), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.encountered_error || self.current_pos >= self.file_length {
            return None;
        }
        let result = self.read_record();
        self.encountered_error = result.is_err();
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.encountered_error || self.current_pos >= self.file_length {
            return (0, Some(0));
        }
        // Records are variable sized, so the .shx count is the only upper bound.
        // If it disagrees with the shapes actually present, fall back to unknown.
        (1, self.remaining.filter(|n| *n > 0))
    }
}

impl<'a, P: GeomProcessor, T: Read + 'a> FusedIterator for ShapeIterator<'a, P, T> {}

pub struct ShapeRecordIterator<'a, P: FeatureProcessor, T: Read + Seek> {
    shape_iter: ShapeIterator<'a, P, T>,
    dbf_reader: dbase::Reader<T>,
    featno: u64,
    encountered_error: bool,
}

pub struct ShapeRecord {
    pub record: dbase::Record,
}

impl<'a, P: FeatureProcessor, T: Read + Seek + 'a> Iterator for ShapeRecordIterator<'a, P, T> {
    type Item = Result<ShapeRecord, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.encountered_error {
            return None;
        }
        if self.featno == 0 {
            self.shape_iter.processor.dataset_begin(None).ok();
        }
        // dbf record iterator does not return None at EOF, so can't drive termination
        if self.shape_iter.current_pos >= self.shape_iter.file_length {
            self.shape_iter.processor.dataset_end().ok();
            return None;
        }
        let result = self.read_feature();
        // shape stream, dbf stream and the processor's event stream are all left mid-feature.
        self.encountered_error = !matches!(result, Some(Ok(_)));
        result
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // A dbf without records yields nothing, whatever the shapes say
        if self.encountered_error || self.dbf_reader.header().num_records == 0 {
            return (0, Some(0));
        }
        // Every feature consumes one shape, and an error ends both iterators.
        // The dbf can still run out first, so there is no lower bound.
        (0, self.shape_iter.size_hint().1)
    }
}

impl<'a, P: FeatureProcessor, T: Read + Seek + 'a> ShapeRecordIterator<'a, P, T> {
    fn read_feature(&mut self) -> Option<Result<ShapeRecord, Error>> {
        let record = match self.dbf_reader.iter_records().next() {
            None => {
                self.shape_iter.processor.dataset_end().ok();
                return None;
            }
            Some(Err(e)) => return Some(Err(Error::DbaseError(e))),
            Some(Ok(rcd)) => rcd,
        };
        let shprec = ShapeRecord { record };

        {
            self.shape_iter.processor.feature_begin(self.featno).ok();
            self.shape_iter.processor.properties_begin().ok();
            if let Err(e) = shprec.process_properties(self.shape_iter.processor) {
                return Some(Err(Error::GeozeroError(e)));
            }
            self.shape_iter.processor.properties_end().ok();

            self.shape_iter.processor.geometry_begin().ok();
        }

        if let Err(e) = self.shape_iter.next()? {
            return Some(Err(e));
        }

        {
            let processor = &mut self.shape_iter.processor;
            processor.geometry_end().ok();
            processor.feature_end(self.featno).ok();
        }
        self.featno += 1;
        Some(Ok(shprec))
    }
}

impl<'a, P: FeatureProcessor, T: Read + Seek + 'a> FusedIterator for ShapeRecordIterator<'a, P, T> {}

/// struct that reads the content of a shapefile
pub struct ShpReader<T: Read + Seek> {
    source: T,
    header: header::Header,
    shapes_index: Option<Vec<ShapeIndex>>,
    dbf_reader: Option<dbase::Reader<T>>,
}

impl<T: Read + Seek> ShpReader<T> {
    /// Creates a new Reader from a source that implements the `Read` trait
    ///
    /// The Shapefile header is read upon creation (but no reading of the Shapes is done)
    ///
    /// # Errors
    ///
    /// Will forward any `std::io::Error`
    ///
    /// Will also return an error if the data is not a shapefile (Wrong file code)
    ///
    /// Will also return an error if the shape type read from the input source is invalid
    pub fn new(mut source: T) -> Result<ShpReader<T>, Error> {
        let header = header::Header::read_from(&mut source)?;

        Ok(ShpReader {
            source,
            header,
            shapes_index: None,
            dbf_reader: None,
        })
    }

    /// Returns a non-mutable reference to the header read
    pub fn header(&self) -> &header::Header {
        &self.header
    }

    /// Read and return _only_ the records contained in the *.dbf* file
    pub fn read_records(self) -> Result<Vec<dbase::Record>, Error> {
        let mut dbf_reader = self.dbf_reader.ok_or(Error::MissingDbf)?;
        dbf_reader.read().map_err(Error::DbaseError)
    }
    ///Return the FieldInfo from the dbf file
    ///Note that the deletion flag is not included in the results
    pub fn dbf_fields(&self) -> Result<Vec<&FieldInfo>, Error> {
        let dbf_reader = self.dbf_reader.as_ref().ok_or(Error::MissingDbf)?;
        //Do not return FieldInfo { Name: DeletionFlag, Field Type: dbase::Character }
        let fields: Vec<_> = dbf_reader
            .fields()
            .iter()
            .filter(|f| f.name() != "DeletionFlag")
            .collect();
        Ok(fields)
    }

    pub fn iter_geometries<P: FeatureProcessor>(
        self,
        processor: &mut P,
    ) -> ShapeIterator<'_, P, T> {
        let file_length = (self.header.file_length as usize).saturating_mul(2);
        // The .shx holds one entry per shape, but it is a separate file: only
        // trust its count when the entries end exactly where the .shp does
        let remaining = self.shapes_index.as_ref().and_then(|index| {
            let last = index.last()?;
            let end = usize::try_from(last.offset)
                .ok()?
                .checked_mul(2)?
                .checked_add(RecordHeader::SIZE)?
                .checked_add(usize::try_from(last.record_size).ok()?.checked_mul(2)?)?;
            (end == file_length).then_some(index.len())
        });
        ShapeIterator {
            processor,
            source: self.source,
            current_pos: header::HEADER_SIZE as usize,
            file_length,
            encountered_error: false,
            remaining,
        }
    }

    /// Returns an iterator over the Shapes and their Records
    ///
    /// # Errors
    ///
    /// The `Result` will be an error if the .dbf wasn't found
    pub fn iter_features<P: FeatureProcessor>(
        mut self,
        processor: &mut P,
    ) -> Result<ShapeRecordIterator<'_, P, T>, Error> {
        let maybe_dbf_reader = self.dbf_reader.take();
        if let Some(dbf_reader) = maybe_dbf_reader {
            let shape_iter = self.iter_geometries(processor);
            Ok(ShapeRecordIterator {
                shape_iter,
                dbf_reader,
                featno: 0,
                encountered_error: false,
            })
        } else {
            Err(Error::MissingDbf)
        }
    }

    /// Reads the index file from the source
    /// This allows to later read shapes by giving their index without reading the whole file
    ///
    /// (see [read_nth_shape()](struct.Reader.html#method.read_nth_shape))
    pub fn add_index_source(&mut self, source: T) -> Result<(), Error> {
        self.shapes_index = Some(read_index_file(source)?);
        Ok(())
    }

    /// Adds the `source` as the source where the dbf record will be read from
    pub fn add_dbf_source(&mut self, source: T) -> Result<(), Error> {
        let dbf_reader = dbase::Reader::new(source)?;
        self.dbf_reader = Some(dbf_reader);
        Ok(())
    }
}

impl ShpReader<BufReader<File>> {
    /// Creates a reader from a path to a file
    ///
    /// Will attempt to read both the .shx and .dbf associated with the file,
    /// if they do not exists the function will not fail, and you will get an error later
    /// if you try to use a function that requires the file to be present.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let shape_path = path.as_ref().to_path_buf();
        let shx_path = shape_path.with_extension("shx");
        let dbf_path = shape_path.with_extension("dbf");

        let source = BufReader::new(File::open(shape_path)?);
        let mut reader = Self::new(source)?;

        if shx_path.exists() {
            let index_source = BufReader::new(File::open(shx_path)?);
            reader.add_index_source(index_source)?;
        }

        if dbf_path.exists() {
            let dbf_source = BufReader::new(File::open(dbf_path)?);
            reader.add_dbf_source(dbf_source)?;
        }
        Ok(reader)
    }
}

// Does not work, because iter_features requires P instead of &mut P
// impl<T: Read> GeozeroDatasource for ShpReader<T> {
//     fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> geozero::error::Result<()> {
//         self.iter_features(*processor).unwrap().all();
//         Ok(())
//     }
// }

// Does not work, because &mut self is required
// impl<P: GeomProcessor, T: Read> GeozeroGeometry for ShapeIterator<P, T> {
//     fn process_geom<G: GeomProcessor>(&self, processor: &mut G) -> geozero::error::Result<()> {
//         if self.current_pos >= self.file_length {
//             Ok(())
//         } else {
//             let hdr = match read_shape(processor, &mut self.source) {
//                 Err(e) => return Ok(()), //FIXME Err(e),
//                 Ok(hdr_and_shape) => hdr_and_shape,
//             };
//             self.current_pos += RecordHeader::SIZE;
//             self.current_pos += hdr.record_size as usize * 2;
//             Ok(())
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::shp::header::{FILE_CODE, HEADER_SIZE, SIZE_OF_SKIP, ShapeType};
    use crate::{ColumnValue, ProcessorSink, PropertyProcessor, error::GeozeroError};

    const POLY_SHP: &str = "./tests/data/shp/poly.shp";
    const POLY_DBF: &str = "./tests/data/shp/poly.dbf";

    /// One shape record: 8-byte header (number, size in 16-bit words, both
    /// big-endian) followed by the shape body.
    fn shape_record(number: i32, size_16_bit: i32, body: &[u8]) -> Vec<u8> {
        let mut r = number.to_be_bytes().to_vec();
        r.extend_from_slice(&size_16_bit.to_be_bytes());
        r.extend_from_slice(body);
        r
    }

    fn point_body(x: f64, y: f64) -> Vec<u8> {
        let mut b = (ShapeType::Point as i32).to_le_bytes().to_vec();
        b.extend_from_slice(&x.to_le_bytes());
        b.extend_from_slice(&y.to_le_bytes());
        b
    }

    /// `file_length` (in 16-bit words) for a shapefile of exactly `bytes` bytes.
    fn words(bytes: usize) -> i32 {
        i32::try_from(bytes / 2).unwrap()
    }

    /// Pull at most `LIMIT` items and return `(ok_count, err_count)`.
    ///
    /// Panics if the iterator has not finished by then, so a non-terminating
    /// iterator fails the test instead of hanging it. Also asserts the iterator
    /// stays exhausted, as its `FusedIterator` impl promises.
    fn drain<I, T, E>(mut it: I) -> (usize, usize)
    where
        I: Iterator<Item = Result<T, E>>,
    {
        const LIMIT: usize = 64;
        let (mut oks, mut errs) = (0, 0);
        for _ in 0..LIMIT {
            match it.next() {
                None => {
                    assert!(it.next().is_none(), "iterator resumed after None");
                    return (oks, errs);
                }
                Some(Ok(_)) => oks += 1,
                Some(Err(_)) => errs += 1,
            }
        }
        panic!("iterator still going after {LIMIT} items ({oks} Ok, {errs} Err)");
    }

    fn geometries(bytes: Vec<u8>) -> (usize, usize) {
        let reader = ShpReader::new(Cursor::new(bytes)).expect("header should parse");
        let mut sink = ProcessorSink::new();
        drain(reader.iter_geometries(&mut sink))
    }

    fn features(shp: Vec<u8>, dbf: Vec<u8>) -> (usize, usize) {
        let mut reader = ShpReader::new(Cursor::new(shp)).expect("header should parse");
        reader.add_dbf_source(Cursor::new(dbf)).unwrap();
        let mut sink = ProcessorSink::new();
        drain(reader.iter_features(&mut sink).unwrap())
    }

    /// A well-formed 100-byte main header with a caller-chosen `file_length`
    fn raw_header(file_length_16_bit: i32) -> Vec<u8> {
        let mut h = Vec::with_capacity(HEADER_SIZE as usize);
        h.extend_from_slice(&FILE_CODE.to_be_bytes());
        h.extend_from_slice(&[0u8; SIZE_OF_SKIP]);
        h.extend_from_slice(&file_length_16_bit.to_be_bytes());
        h.extend_from_slice(&1000i32.to_le_bytes()); // version
        h.extend_from_slice(&(ShapeType::Polygon as i32).to_le_bytes());
        h.extend_from_slice(&[0u8; 64]); // 8 x f64 bbox
        assert_eq!(h.len(), HEADER_SIZE as usize);
        h
    }

    #[test]
    fn truncated_file_reports_one_error_and_stops() {
        // The header is entirely well formed and claims 1000 bytes; the file is
        // the 100-byte header alone. Every truncated download looks like this.
        assert_eq!(geometries(raw_header(500)), (0, 1));
    }

    #[test]
    fn truncation_after_a_valid_record_reports_one_error_and_stops() {
        let mut bytes = raw_header(5000);
        bytes.extend(shape_record(1, 10, &point_body(1.0, 2.0)));
        assert_eq!(geometries(bytes), (1, 1));
    }

    #[test]
    fn invalid_shape_type_reports_one_error_and_stops() {
        let mut body = 60i32.to_le_bytes().to_vec(); // not a ShapeType
        body.extend_from_slice(&[0u8; 16]);
        let mut bytes = raw_header(5000);
        bytes.extend(shape_record(1, 10, &body));
        assert_eq!(geometries(bytes), (0, 1));
    }

    #[test]
    fn negative_record_size_is_rejected_not_panicked() {
        let mut bytes = raw_header(5000);
        bytes.extend(shape_record(1, -1, &[0u8; 32]));
        assert_eq!(geometries(bytes), (0, 1));
    }

    #[test]
    fn huge_file_length_does_not_overflow_the_iteration_extent() {
        assert_eq!(geometries(raw_header(i32::MAX)), (0, 1));
    }

    #[test]
    fn header_only_file_yields_no_geometries() {
        assert_eq!(geometries(raw_header(HEADER_SIZE / 2)), (0, 0));
    }

    #[test]
    fn well_formed_file_yields_every_record() {
        let records: Vec<u8> = (1..=3)
            .flat_map(|n| shape_record(n, 10, &point_body(f64::from(n), 2.0)))
            .collect();
        let mut bytes = raw_header(words(HEADER_SIZE as usize + records.len()));
        bytes.extend(records);
        assert_eq!(geometries(bytes), (3, 0));
    }

    #[test]
    fn truncated_shp_stops_the_feature_iterator() {
        let shp = std::fs::read(POLY_SHP).unwrap();
        let dbf = std::fs::read(POLY_DBF).unwrap();
        assert_eq!(features(shp[..600].to_vec(), dbf), (1, 1));
    }

    #[test]
    fn truncated_dbf_stops_the_feature_iterator() {
        let shp = std::fs::read(POLY_SHP).unwrap();
        let dbf = std::fs::read(POLY_DBF).unwrap();
        let (_, errs) = features(shp, dbf[..dbf.len() / 2].to_vec());
        assert_eq!(errs, 1);
    }

    #[test]
    fn processor_error_stops_the_feature_iterator() {
        struct FailingProperties;
        impl GeomProcessor for FailingProperties {}
        impl FeatureProcessor for FailingProperties {}
        impl PropertyProcessor for FailingProperties {
            fn property(
                &mut self,
                _idx: usize,
                _name: &str,
                _value: &ColumnValue,
            ) -> crate::error::Result<bool> {
                Err(GeozeroError::Property("rejected".to_string()))
            }
        }

        let shp = std::fs::read(POLY_SHP).unwrap();
        let dbf = std::fs::read(POLY_DBF).unwrap();
        let mut reader = ShpReader::new(Cursor::new(shp)).unwrap();
        reader.add_dbf_source(Cursor::new(dbf)).unwrap();
        let mut processor = FailingProperties;
        assert_eq!(drain(reader.iter_features(&mut processor).unwrap()), (0, 1));
    }

    #[test]
    fn well_formed_shapefile_yields_every_feature() {
        let shp = std::fs::read(POLY_SHP).unwrap();
        let dbf = std::fs::read(POLY_DBF).unwrap();
        assert_eq!(features(shp, dbf), (10, 0));
    }
}
