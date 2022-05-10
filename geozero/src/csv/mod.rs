//! CSV conversions.
pub(crate) mod csv_reader;
pub(crate) mod csv_writer;

pub use csv_reader::*;
pub use csv_writer::*;

pub(crate) mod conversion {
    use super::csv_writer::*;
    use crate::error::Result;
    use crate::GeozeroDatasource;

    /// Consume features into CSV
    pub trait ProcessToCsv {
        /// Consume features into CSV String.
        fn to_csv(&mut self) -> Result<String>;
    }

    impl<T: GeozeroDatasource> ProcessToCsv for T {
        fn to_csv(&mut self) -> Result<String> {
            let mut out: Vec<u8> = Vec::new();
            {
                let mut p = CsvWriter::new(&mut out);
                self.process(&mut p)?;
            }
            String::from_utf8(out).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })
        }
    }
}
