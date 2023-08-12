//! Zero-Copy reading and writing of geospatial data.
//!
//! `GeoZero` defines an API for reading geospatial data formats without an intermediate representation.
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
//! |           |                         [`GeozeroGeometry`]                                                                              | Dimensions |                        [`GeozeroDatasource`]                                         | Geometry Conversion |            [`GeomProcessor`]            |
//! |-----------|--------------------------------------------------------------------------------------------------------------------------|------------|--------------------------------------------------------------------------------------|---------------------|-----------------------------------------|
//! | CSV       | [csv::Csv], [csv::CsvString]                                                                                             | XY         | -                                                                                    | [ProcessToCsv]      | [CsvWriter](csv::CsvWriter)             |
//! | GDAL      | `gdal::vector::Geometry`                                                                                                 | XYZ        | -                                                                                    | [ToGdal]            | [GdalWriter](gdal::GdalWriter)          |
//! | geo-types | `geo_types::Geometry<f64>`                                                                                               | XY         | -                                                                                    | [ToGeo]             | [GeoWriter](geo_types::GeoWriter)       |
//! | GeoArrow  | `arrow2::array::BinaryArray`                                                                                             | XY         | -                                                                                    | -                   | -                                       |
//! | GeoJSON   | [GeoJson](geojson::GeoJson), [GeoJsonString](geojson::GeoJsonString)                                                     | XYZ        | [GeoJsonReader](geojson::GeoJsonReader), [GeoJson](geojson::GeoJson)                 | [ToJson]            | [GeoJsonWriter](geojson::GeoJsonWriter) |
//! | GEOS      | `geos::Geometry`                                                                                                         | XYZ        | -                                                                                    | [ToGeos]            | [GeosWriter](geos::GeosWriter)          |
//! | GPX       |                                                                                                                          | XY         | [GpxReader](gpx::GpxReader)                                                          |                     |                                         |
//! | MVT       | [mvt::tile::Feature]                                                                                                     | XY         | [mvt::tile::Layer]                                                                   | [ToMvt]             | [MvtWriter](mvt::MvtWriter)             |
//! | SVG       | -                                                                                                                        | XY         | -                                                                                    | [ToSvg]             | [SvgWriter](svg::SvgWriter)             |
//! | WKB       | [Wkb](wkb::Wkb), [Ewkb](wkb::Ewkb), [GpkgWkb](wkb::GpkgWkb), [SpatiaLiteWkb](wkb::SpatiaLiteWkb), [MySQL](wkb::MySQLWkb) | XYZM       | -                                                                                    | [ToWkb]             | [WkbWriter](wkb::WkbWriter)             |
//! | WKT       | [wkt::WktStr], [wkt::WktString], [wkt::EwktStr], [wkt::EwktString]                                                       | XYZM       | [wkt::WktReader], [wkt::WktStr], [wkt::WktString], [wkt::EwktStr], [wkt::EwktString] | [ToWkt]             | [WktWriter](wkt::WktWriter)             |

#![warn(clippy::uninlined_format_args)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::many_single_char_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::redundant_closure_for_method_calls,
    clippy::similar_names,
    clippy::struct_excessive_bools
)]

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

#[cfg(feature = "with-arrow")]
pub mod arrow;

#[cfg(feature = "with-csv")]
pub mod csv;
#[cfg(feature = "with-csv")]
pub use crate::csv::conversion::*;

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

#[cfg(feature = "with-gpx")]
pub mod gpx;

#[cfg(any(
    feature = "with-postgis-diesel",
    feature = "with-postgis-postgres",
    feature = "with-postgis-sqlx",
))]
pub mod postgis;

#[cfg(feature = "with-svg")]
pub mod svg;
#[cfg(feature = "with-svg")]
pub use crate::svg::conversion::*;

#[cfg(feature = "with-tessellator")]
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
#[derive(Default)]
pub struct ProcessorSink;

impl ProcessorSink {
    pub fn new() -> Self {
        Self
    }
}

impl FeatureProcessor for ProcessorSink {}
impl GeomProcessor for ProcessorSink {}
impl PropertyProcessor for ProcessorSink {}
