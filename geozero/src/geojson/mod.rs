//! GeoJSON conversions.
pub(crate) mod geojson_reader;
pub(crate) mod geojson_writer;

pub use geojson_reader::*;
pub use geojson_writer::*;

pub(crate) mod conversion {
    use super::geojson_writer::*;
    use crate::error::Result;
    use crate::{GeozeroDatasource, GeozeroGeometry};

    /// Convert to GeoJSON.
    pub trait ToJson {
        /// Convert to GeoJSON String.
        fn to_json(&self) -> Result<String>;
    }

    impl<T: GeozeroGeometry> ToJson for T {
        fn to_json(&self) -> Result<String> {
            let mut out: Vec<u8> = Vec::new();
            let mut p = GeoJsonWriter::new(&mut out);
            self.process_geom(&mut p)?;
            String::from_utf8(out).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })
        }
    }

    /// Consume features as GeoJSON.
    pub trait ProcessToJson {
        /// Consume features as GeoJSON String.
        fn to_json(&mut self) -> Result<String>;
    }

    impl<T: GeozeroDatasource> ProcessToJson for T {
        fn to_json(&mut self) -> Result<String> {
            let mut out: Vec<u8> = Vec::new();
            let mut p = GeoJsonWriter::new(&mut out);
            self.process(&mut p)?;
            String::from_utf8(out).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })
        }
    }
}
