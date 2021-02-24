//! geo-types conversions.
pub(crate) mod geo_types_reader;
pub(crate) mod geo_types_writer;

pub use geo_types_reader::*;
pub use geo_types_writer::*;

pub(crate) mod conversion {
    use super::geo_types_writer::*;
    use crate::error::Result;
    use crate::GeozeroGeometry;

    /// Convert to geo-types Geometry.
    pub trait ToGeo {
        /// Convert to geo-types Geometry.
        fn to_geo(&self) -> Result<geo_types::Geometry<f64>>;
    }

    impl<T: GeozeroGeometry> ToGeo for T {
        fn to_geo(&self) -> Result<geo_types::Geometry<f64>> {
            let mut geo = GeoWriter::new();
            self.process_geom(&mut geo)?;
            Ok(geo.geom)
        }
    }
}

#[cfg(feature = "with-wkb")]
mod wkb {
    use super::geo_types_writer::*;
    use crate::error::Result;
    use crate::wkb::{FromWkb, WkbDialect};
    use std::io::Read;

    impl FromWkb for geo_types::Geometry<f64> {
        fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> Result<Self> {
            let mut geo = GeoWriter::new();
            crate::wkb::process_wkb_type_geom(rdr, &mut geo, dialect)?;
            Ok(geo.geom)
        }
    }
}
