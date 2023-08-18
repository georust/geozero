//! Well-Known Binary (WKB) conversions.
//!
//! # Usage example:
//!
//! Convert a EWKB geometry to WKT:
//!
//! ```
//! use geozero::{ToWkt, wkb::Ewkb};
//!
//! let wkb = Ewkb(vec![1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 64, 0, 0, 0, 0, 0, 0, 52, 192]);
//! assert_eq!(wkb.to_wkt().unwrap(), "POINT(10 -20)");
//! ```
pub(crate) mod wkb_common;
pub(crate) mod wkb_reader;
pub(crate) mod wkb_writer;

pub use wkb_common::*;
pub use wkb_reader::*;
pub use wkb_writer::*;

pub(crate) mod conversion {
    use crate::error::Result;
    use crate::wkb::{WkbDialect, WkbWriter};
    use crate::{CoordDimensions, GeozeroGeometry};

    /// Convert to WKB.
    ///
    /// # Usage example:
    ///
    /// Convert a geo-types `Point` to EWKB:
    ///
    /// ```
    /// use geozero::{CoordDimensions, ToWkb};
    ///
    /// let geom: geo_types::Geometry<f64> = geo_types::Point::new(10.0, -20.0).into();
    /// let wkb = geom.to_ewkb(CoordDimensions::xy(), Some(4326)).unwrap();
    /// assert_eq!(&wkb, &[1, 1, 0, 0, 32, 230, 16, 0, 0, 0, 0, 0, 0, 0, 0, 36, 64, 0, 0, 0, 0, 0, 0, 52, 192]);
    /// ```
    pub trait ToWkb {
        /// Convert to WKB dialect.
        fn to_wkb_dialect(
            &self,
            dialect: WkbDialect,
            dims: CoordDimensions,
            srid: Option<i32>,
            envelope: Vec<f64>,
        ) -> Result<Vec<u8>>;
        /// Convert to OGC WKB.
        fn to_wkb(&self, dims: CoordDimensions) -> Result<Vec<u8>> {
            self.to_wkb_dialect(WkbDialect::Wkb, dims, None, Vec::new())
        }
        /// Convert to EWKB.
        fn to_ewkb(&self, dims: CoordDimensions, srid: Option<i32>) -> Result<Vec<u8>> {
            self.to_wkb_dialect(WkbDialect::Ewkb, dims, srid, Vec::new())
        }
        /// Convert to GeoPackage WKB.
        fn to_gpkg_wkb(
            &self,
            dims: CoordDimensions,
            srid: Option<i32>,
            envelope: Vec<f64>,
        ) -> Result<Vec<u8>> {
            self.to_wkb_dialect(WkbDialect::Geopackage, dims, srid, envelope)
        }
        /// Convert to Spatialite WKB.
        fn to_spatialite_wkb(
            &self,
            dims: CoordDimensions,
            srid: Option<i32>,
            envelope: Vec<f64>,
        ) -> Result<Vec<u8>> {
            self.to_wkb_dialect(WkbDialect::SpatiaLite, dims, srid, envelope)
        }
        /// Convert to MySQL WKB.
        fn to_mysql_wkb(&self, srid: Option<i32>) -> Result<Vec<u8>> {
            self.to_wkb_dialect(
                WkbDialect::MySQL,
                CoordDimensions::default(),
                srid,
                Vec::new(),
            )
        }
    }

    impl<T: GeozeroGeometry> ToWkb for T {
        fn to_wkb_dialect(
            &self,
            dialect: WkbDialect,
            dims: CoordDimensions,
            srid: Option<i32>,
            envelope: Vec<f64>,
        ) -> Result<Vec<u8>> {
            let mut wkb: Vec<u8> = Vec::new();
            let mut writer = WkbWriter::with_opts(&mut wkb, dialect, dims, srid, envelope);
            self.process_geom(&mut writer)?;
            Ok(wkb)
        }
    }
}
