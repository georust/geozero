//! Collection of GeoZero API implementations.

#[cfg(feature = "gdal-lib")]
mod gdal_reader;
#[cfg(feature = "gdal-lib")]
mod gdal_writer;
mod geo_types_writer;
mod geojson_reader;
mod geojson_writer;
#[cfg(feature = "gpkg")]
mod geopackage;
#[cfg(feature = "geos-lib")]
mod geos_reader;
#[cfg(feature = "geos-lib")]
mod geos_writer;
/// PostGIS geometry type conversions.
pub mod postgis;
/// SVG Writer.
pub mod svg;
mod wkb_common;
mod wkb_reader;
mod wkb_writer;
mod wkt_writer;

/// GeoJSON Reader + Writer.
pub mod geojson {
    pub use crate::geojson_reader::*;
    pub use crate::geojson_writer::*;
}

/// [geo-types](https://github.com/georust/geo) Writer.
pub mod geo_types {
    pub use crate::geo_types_writer::*;
}

/// WKB Reader.
pub mod wkb {
    pub use crate::wkb_common::*;
    pub use crate::wkb_reader::*;
    pub use crate::wkb_writer::*;
}

/// WKT Writer.
pub mod wkt {
    pub use crate::wkt_writer::*;
}

/// [GEOS](https://github.com/georust/geos) Reader + Writer.
#[cfg(feature = "geos-lib")]
pub mod geos {
    pub use crate::geos_reader::*;
    pub use crate::geos_writer::*;
}

/// [GDAL](https://github.com/georust/gdal) Reader + Writer.
#[cfg(feature = "gdal-lib")]
pub mod gdal {
    pub use crate::gdal_reader::*;
    pub use crate::gdal_writer::*;
}

/// Type conversions for [SQLx](https://github.com/launchbadge/sqlx)
#[cfg(feature = "gpkg")]
pub mod gpkg {
    pub use crate::geopackage::*;
}
