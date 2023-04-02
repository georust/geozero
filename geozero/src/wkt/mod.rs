//! Well-Known Text (WKT) conversions.
//!
//! OpenGIS Simple Features Specification For SQL Revision 1.1, Chapter 3.2.5
pub(crate) mod wkt_reader;
pub(crate) mod wkt_writer;

pub use wkt_reader::*;
pub use wkt_writer::*;

pub(crate) mod conversion {
    use crate::error::Result;
    use crate::wkt::WktWriter;
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

#[cfg(feature = "with-wkb")]
mod wkb {
    use crate::error::Result;
    use crate::wkb::{FromWkb, WkbDialect};
    use crate::wkt::{WktString, WktWriter};
    use std::io::Read;

    impl FromWkb for WktString {
        fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> Result<Self> {
            let mut out: Vec<u8> = Vec::new();
            let mut writer = WktWriter::new(&mut out);
            crate::wkb::process_wkb_type_geom(rdr, &mut writer, dialect)?;
            let wkt = String::from_utf8(out).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })?;
            Ok(WktString(wkt))
        }
    }
}
