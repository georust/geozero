//! Zero-Copy reading and writing of geospatial data.
//!
//! GeoZero defines an API for reading geospatial data formats without an intermediate representation.
//! It defines traits which can be implemented to read and convert to an arbitrary format
//! or render geometries directly.
//!
//! Supported geometry types:
//! * [OGC Simple Features](https://en.wikipedia.org/wiki/Simple_Features)
//! * Circular arcs as defined by SQL-MM Part 3
//! * TIN
//!
//! Supported dimensions: X, Y, Z, M, T
//!
//! Available implementations:
//! * [geozero-shp](https://docs.rs/geozero-shp)
//! * [flatgeobuf](https://docs.rs/flatgeobuf)
//!
//! ## Format conversion overview
//!
//! |           |                          [GeozeroGeometry]                           | Dimensions |                         [GeozeroDatasource]                          | Geometry Conversion |             [GeomProcessor]             |
//! |-----------|----------------------------------------------------------------------|------------|----------------------------------------------------------------------|---------------------|-----------------------------------------|
//! | geo-types | `geo_types::Geometry<f64>`                                           | XY         | -                                                                    | [ToGeo]             | [GeoWriter](geo_types::GeoWriter)       |
//! | GeoJSON   | [GeoJson](geojson::GeoJson), [GeoJsonString](geojson::GeoJsonString) | XYZ        | [GeoJsonReader](geojson::GeoJsonReader), [GeoJson](geojson::GeoJson) | [ToJson]            | [GeoJsonWriter](geojson::GeoJsonWriter) |
//! | GDAL      | `gdal::vector::Geometry`                                             | XYZ        | -                                                                    | [ToGdal]            | [GdalWriter](gdal::GdalWriter)          |
//! | GEOS      | `geos::Geometry`                                                     | XYZ        | -                                                                    | [ToGeos]            | [GeosWriter](geos::GeosWriter)          |
//! | SVG       | -                                                                    | XY         | -                                                                    | [ToSvg]             | [SvgWriter](svg::SvgWriter)             |
//! | WKB       | [Wkb](wkb::Wkb), [Ewkb](wkb::Ewkb), [GpkgWkb](wkb::GpkgWkb)          | XYZM       | -                                                                    | [ToWkb]             | [WkbWriter](wkb::WkbWriter)             |
//! | WKT       | -                                                                    | XYZM       | -                                                                    | [ToWkt]             | [WktWriter](wkt::WktWriter)             |

mod api;
pub mod error;
mod feature_processor;
mod geometry_processor;
mod multiplex;
mod property_processor;

pub use api::*;
pub use feature_processor::*;
pub use geometry_processor::*;
pub use multiplex::*;
pub use property_processor::*;

#[cfg(feature = "with-gdal")]
pub mod gdal;
#[cfg(feature = "with-gdal")]
pub use crate::gdal::conversion::*;

#[cfg(feature = "with-geo")]
pub mod geo_types;
#[cfg(feature = "with-geo")]
pub use crate::geo_types::conversion::*;

#[cfg(feature = "with-geojson")]
pub mod geojson;
#[cfg(feature = "with-geojson")]
pub use crate::geojson::conversion::*;

#[cfg(feature = "with-geos")]
pub mod geos;
#[cfg(feature = "with-geos")]
pub use crate::geos::conversion::*;

#[cfg(feature = "with-gpkg")]
pub mod gpkg;

#[cfg(any(feature = "with-postgis-postgres", feature = "with-postgis-sqlx"))]
pub mod postgis;

#[cfg(feature = "with-svg")]
pub mod svg;
#[cfg(feature = "with-svg")]
pub use crate::svg::conversion::*;

#[cfg(feature = "with-tesselator")]
pub mod tessellator;

#[cfg(feature = "with-wkb")]
pub mod wkb;
#[cfg(feature = "with-wkb")]
pub use crate::wkb::conversion::*;

#[cfg(feature = "with-wkt")]
pub mod wkt;
#[cfg(feature = "with-wkt")]
pub use crate::wkt::conversion::*;

#[cfg(feature = "with-mvt")]
pub mod mvt;
#[cfg(feature = "with-mvt")]
pub use crate::mvt::conversion::*;

/// Empty processor implementation
pub struct ProcessorSink;

impl ProcessorSink {
    pub fn new() -> ProcessorSink {
        ProcessorSink {}
    }
}

impl feature_processor::FeatureProcessor for ProcessorSink {}
impl geometry_processor::GeomProcessor for ProcessorSink {}
impl property_processor::PropertyProcessor for ProcessorSink {}
