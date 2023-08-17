//! GDAL conversions.
mod gdal_error;
pub(crate) mod gdal_reader;
pub(crate) mod gdal_writer;

pub use gdal_error::GdalError;
pub use gdal_reader::*;
pub use gdal_writer::*;

pub(crate) mod conversion {
    use crate::error::Result;
    use crate::gdal::GdalWriter;
    use crate::{CoordDimensions, GeozeroGeometry};
    use gdal::vector::Geometry;

    /// Convert to GDAL geometry.
    pub trait ToGdal {
        /// Convert to 2D GDAL geometry.
        fn to_gdal(&self) -> Result<Geometry>;
        /// Convert to GDAL geometry with dimensions.
        fn to_gdal_ndim(&self, dims: CoordDimensions) -> Result<Geometry>;
    }

    impl<T: GeozeroGeometry> ToGdal for T {
        fn to_gdal(&self) -> Result<Geometry> {
            self.to_gdal_ndim(CoordDimensions::default())
        }
        fn to_gdal_ndim(&self, dims: CoordDimensions) -> Result<Geometry> {
            let mut gdal = GdalWriter::with_dims(dims);
            self.process_geom(&mut gdal)?;
            Ok(gdal.geom)
        }
    }
}

#[cfg(feature = "with-wkb")]
mod wkb {
    use crate::error::Result;
    use crate::gdal::GdalWriter;
    use crate::wkb::{FromWkb, WkbDialect};
    use std::io::Read;

    impl FromWkb for gdal::vector::Geometry {
        fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> Result<Self> {
            let mut gdal = GdalWriter::new();
            crate::wkb::process_wkb_type_geom(rdr, &mut gdal, dialect)?;
            Ok(gdal.geom)
        }
    }
}
