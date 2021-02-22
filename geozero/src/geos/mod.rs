//! [GEOS](https://github.com/georust/geos) conversions.
pub(crate) mod geos_reader;
pub(crate) mod geos_writer;

pub use geos_reader::*;
pub use geos_writer::*;

pub(crate) mod conversion {
    use super::geos_writer::*;
    use crate::error::Result;
    use crate::wkb::{FromWkb, WkbDialect};
    use crate::GeozeroGeometry;
    use std::io::Read;

    /// Convert to GEOS geometry.
    pub trait ToGeos {
        /// Convert to GEOS geometry.
        fn to_geos(&self) -> Result<geos::Geometry<'_>>;
    }

    impl<T: GeozeroGeometry> ToGeos for T {
        fn to_geos(&self) -> Result<geos::Geometry<'_>> {
            let mut geos = GeosWriter::new();
            self.process_geom(&mut geos)?;
            Ok(geos.geom)
        }
    }

    impl FromWkb for geos::Geometry<'_> {
        fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> Result<Self> {
            let mut geos = GeosWriter::new();
            crate::wkb::process_wkb_type_geom(rdr, &mut geos, dialect)?;
            Ok(geos.geom)
        }
    }
}
