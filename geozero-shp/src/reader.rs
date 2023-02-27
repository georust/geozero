use crate::shp_reader::{read_shape, RecordHeader};
use crate::shx_reader::{read_index_file, ShapeIndex};
use crate::{header, Error};
pub use dbase::{FieldInfo, FieldType};
use geozero::{FeatureProcessor, FeatureProperties, GeomProcessor};
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::iter::FusedIterator;
use std::path::Path;

/// Struct that handle iteration over the shapes of a .shp file
pub struct ShapeIterator<'a, P: GeomProcessor, T: Read> {
    processor: &'a mut P,
    source: T,
    current_pos: usize,
    file_length: usize,
}

impl<'a, P: GeomProcessor, T: Read + 'a> Iterator for ShapeIterator<'a, P, T> {
    type Item = Result<(), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos >= self.file_length {
            None
        } else {
            let hdr = match read_shape(self.processor, &mut self.source) {
                Err(e) => return Some(Err(e)),
                Ok(hdr_and_shape) => hdr_and_shape,
            };
            self.current_pos += RecordHeader::SIZE;
            self.current_pos += hdr.record_size as usize * 2;
            Some(Ok(()))
        }
    }
}

impl<'a, P: FeatureProcessor, T: Read + Seek + 'a> FusedIterator for ShapeIterator<'a, P, T> {}

pub struct ShapeRecordIterator<'a, P: FeatureProcessor, T: Read + Seek> {
    shape_iter: ShapeIterator<'a, P, T>,
    dbf_reader: dbase::Reader<T>,
    featno: u64,
}

pub struct ShapeRecord {
    pub record: dbase::Record,
}

impl<'a, P: FeatureProcessor, T: Read + Seek + 'a> Iterator for ShapeRecordIterator<'a, P, T> {
    type Item = Result<ShapeRecord, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.featno == 0 {
            self.shape_iter.processor.dataset_begin(None).ok();
        }
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
pub struct Reader<T: Read + Seek> {
    source: T,
    header: header::Header,
    shapes_index: Option<Vec<ShapeIndex>>,
    dbf_reader: Option<dbase::Reader<T>>,
}

impl<T: Read + Seek> Reader<T> {
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
    /// Will also return an error if the shapetype read from the input source is invalid
    pub fn new(mut source: T) -> Result<Reader<T>, Error> {
        let header = header::Header::read_from(&mut source)?;

        Ok(Reader {
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

    pub fn iter_geometries<P: FeatureProcessor>(self, processor: &mut P) -> ShapeIterator<P, T> {
        ShapeIterator {
            processor,
            source: self.source,
            current_pos: header::HEADER_SIZE as usize,
            file_length: (self.header.file_length * 2) as usize,
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
    ) -> Result<ShapeRecordIterator<P, T>, Error> {
        let maybe_dbf_reader = self.dbf_reader.take();
        if let Some(dbf_reader) = maybe_dbf_reader {
            let shape_iter = self.iter_geometries(processor);
            Ok(ShapeRecordIterator {
                shape_iter,
                dbf_reader,
                featno: 0,
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

impl Reader<BufReader<File>> {
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
// impl<T: Read> GeozeroDatasource for Reader<T> {
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
