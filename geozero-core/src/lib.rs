//! Collection of GeoZero API implementations.

#[cfg(feature = "gdal-lib")]
mod gdal_reader;
#[cfg(feature = "gdal-lib")]
mod gdal_writer;
mod geo_types_reader;
mod geo_types_writer;
mod geojson_reader;
mod geojson_writer;
#[cfg(feature = "gpkg")]
mod geopackage;
#[cfg(feature = "geos-lib")]
mod geos_reader;
#[cfg(feature = "geos-lib")]
mod geos_writer;
mod geozero;
/// PostGIS geometry type encoding/decoding.
pub mod postgis;
/// SVG conversions.
pub mod svg;
#[cfg(feature = "tesselator")]
pub mod tessellator;
mod wkb_common;
mod wkb_reader;
mod wkb_writer;
mod wkt_writer;

pub use crate::geozero::*;

/// GeoJSON conversions.
pub mod geojson {
    pub use crate::geojson_reader::*;
    pub use crate::geojson_writer::*;
}
pub use crate::geojson_writer::conversion::*;

/// [geo-types](https://github.com/georust/geo) conversions.
pub mod geo_types {
    pub use crate::geo_types_reader::*;
    pub use crate::geo_types_writer::*;
}
pub use crate::geo_types_writer::conversion::*;

/// Well-Known Binary (WKB) conversions.
pub mod wkb {
    pub use crate::wkb_common::*;
    pub use crate::wkb_reader::*;
    pub use crate::wkb_writer::*;
}

/// Well-Known Text (WKT) conversions.
///
/// OpenGIS Simple Features Specification For SQL Revision 1.1, Chapter 3.2.5
pub mod wkt {
    pub use crate::wkt_writer::*;
}
pub use crate::wkt_writer::conversion::*;

/// [GEOS](https://github.com/georust/geos) conversions.
#[cfg(feature = "geos-lib")]
pub mod geos {
    pub use crate::geos_reader::*;
    pub use crate::geos_writer::*;
}
#[cfg(feature = "geos-lib")]
pub use crate::geos_writer::conversion::*;

/// [GDAL](https://github.com/georust/gdal) conversions.
#[cfg(feature = "gdal-lib")]
pub mod gdal {
    pub use crate::gdal_reader::*;
    pub use crate::gdal_writer::*;
}

/// Geopackage geometry type encoding/decoding.
#[cfg(feature = "gpkg")]
pub mod gpkg {
    pub use crate::geopackage::*;
}

pub use crate::svg::conversion::*;
