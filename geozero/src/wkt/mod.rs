//! Well-Known Text (WKT) conversions.
//!
//! OpenGIS Simple Features Specification For SQL Revision 1.1, Chapter 3.2.5
pub(crate) mod wkt_writer;

pub use wkt_writer::*;

pub(crate) mod conversion {
    use super::wkt_writer::*;
    use crate::error::Result;
    use crate::{CoordDimensions, GeozeroGeometry};

    /// Convert to WKT.
    pub trait ToWkt {
        /// Convert to 2D WKT String.
        fn to_wkt(&self) -> Result<String>;
        /// Convert to WKT String with dimensions.
        fn to_wkt_ndim(&self, dims: CoordDimensions) -> Result<String>;
    }

    impl<T: GeozeroGeometry> ToWkt for T {
        fn to_wkt(&self) -> Result<String> {
            self.to_wkt_ndim(CoordDimensions::default())
        }
        fn to_wkt_ndim(&self, dims: CoordDimensions) -> Result<String> {
            let mut out: Vec<u8> = Vec::new();
            let mut writer = WktWriter::new(&mut out);
            writer.dims = dims;
            self.process_geom(&mut writer)?;
            String::from_utf8(out).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })
        }
    }
}
