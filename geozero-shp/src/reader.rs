use crate::shp_reader::{read_one_shape_as, RecordHeader};
use crate::shx_reader::{read_index_file, ShapeIndex};
use crate::{header, Error};
use geozero::GeomProcessor;
use std::fs::File;
use std::io::{BufReader, Read};
use std::iter::FusedIterator;
use std::path::Path;

#[derive(Debug)]
enum IteratorState {
    Initial,     // Header read
    RecordBegin, // Ready to read a record
    Reading,
    RecordEnd, // At end of record
}
/// Struct that handle iteration over the shapes of a .shp file
pub struct ShapeIterator<T: Read> {
    state: IteratorState,
    source: T,
    current_pos: usize,
    file_length: usize,
}

impl<T: Read> ShapeIterator<T> {
    fn process_shape<P: GeomProcessor>(&mut self, processor: &mut P) -> geozero::error::Result<()> {
        self.state = IteratorState::Reading;
        if let Ok(hdr) = read_one_shape_as(processor, &mut self.source) {
            self.current_pos += RecordHeader::SIZE;
            self.current_pos += hdr.record_size as usize * 2;
            self.state = IteratorState::RecordEnd;
        }
        Ok(())
    }
    fn skip_record(&mut self) -> Result<(), Error> {
        self.state = IteratorState::Reading;
        let hdr = RecordHeader::read_from(&mut self.source)?;
        let record_size = (hdr.record_size * 2) as usize;
        let mut buffer = vec![0; record_size];
        self.source.read_exact(&mut buffer).unwrap();
        self.current_pos += RecordHeader::SIZE + record_size;
        self.state = IteratorState::RecordEnd;
        Ok(())
    }
}

impl<T: Read> ShapeIterator<T> {
    /// Consume and process geometry.
    pub fn process_geom<P: GeomProcessor>(
        &mut self,
        processor: &mut P,
    ) -> geozero::error::Result<()> {
        self.process_shape(processor)
    }
}

impl<T: Read> Iterator for ShapeIterator<T> {
    type Item = Result<(), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos >= self.file_length {
            None
        } else {
            match self.state {
                IteratorState::Initial => {}
                IteratorState::RecordBegin => {
                    self.skip_record().unwrap(); //map err
                    if self.current_pos >= self.file_length {
                        return None;
                    }
                }
                IteratorState::RecordEnd => {}
                IteratorState::Reading => {
                    return None; // FIXME: Some(Err(...))
                }
            };
            self.state = IteratorState::RecordBegin;
            Some(Ok(()))
        }
    }
}

impl<T: Read> FusedIterator for ShapeIterator<T> {}

pub struct ShapeRecordIterator<T: Read> {
    shape_iter: ShapeIterator<T>,
    dbf_reader: dbase::Reader<T>,
}

pub struct ShapeRecord {
    pub record: dbase::Record,
}

impl<T: Read> Iterator for ShapeRecordIterator<T> {
    type Item = Result<ShapeRecord, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let _shape = match self.shape_iter.next()? {
            Err(e) => return Some(Err(e)),
            Ok(shp) => shp,
        };

        let record = match self.dbf_reader.next()? {
            Err(e) => return Some(Err(Error::DbaseError(e))),
            Ok(rcd) => rcd,
        };

        Some(Ok(ShapeRecord { record }))
    }
}

impl<T: Read> FusedIterator for ShapeRecordIterator<T> {}

/// struct that reads the content of a shapefile
pub struct Reader<T: Read> {
    source: T,
    header: header::Header,
    shapes_index: Option<Vec<ShapeIndex>>,
    dbf_reader: Option<dbase::Reader<T>>,
}

impl<T: Read> Reader<T> {
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
        let dbf_reader = self.dbf_reader.ok_or(Error::MissingDbf)?;
        dbf_reader.read().or_else(|e| Err(Error::DbaseError(e)))
    }

    pub fn iter_geometries(self) -> ShapeIterator<T> {
        ShapeIterator {
            state: IteratorState::Initial,
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
    pub fn iter_features(mut self) -> Result<ShapeRecordIterator<T>, Error> {
        let maybe_dbf_reader = self.dbf_reader.take();
        if let Some(dbf_reader) = maybe_dbf_reader {
            let shape_iter = self.iter_geometries();
            Ok(ShapeRecordIterator {
                shape_iter,
                dbf_reader,
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

impl<T: Read> IntoIterator for Reader<T> {
    type Item = Result<(), Error>;
    type IntoIter = ShapeIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_geometries()
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
