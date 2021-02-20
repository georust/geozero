//! Collection of GeoZero API implementations.

#[cfg(feature = "gdal-lib")]
pub(crate) mod gdal_reader;
#[cfg(feature = "gdal-lib")]
pub(crate) mod gdal_writer;
pub(crate) mod geo_types_reader;
pub(crate) mod geo_types_writer;
pub(crate) mod geojson_reader;
pub(crate) mod geojson_writer;
#[cfg(feature = "gpkg")]
pub(crate) mod geopackage;
#[cfg(feature = "geos-lib")]
pub(crate) mod geos_reader;
#[cfg(feature = "geos-lib")]
pub(crate) mod geos_writer;
/// PostGIS geometry type encoding/decoding.
pub mod postgis;
/// SVG conversions.
pub mod svg;
#[cfg(feature = "tesselator")]
pub mod tessellator;
pub(crate) mod wkb_common;
pub(crate) mod wkb_reader;
pub(crate) mod wkb_writer;
pub(crate) mod wkt_writer;
