#![cfg_attr(feature = "default", doc = include_str!("../../README.md"))]

mod api;
pub mod error;
mod feature_processor;
mod geometry_processor;
mod multiplex;
mod property_processor;
mod wrap;

pub use api::*;
pub use feature_processor::*;
pub use geometry_processor::*;
pub use multiplex::*;
pub use property_processor::*;
pub use wrap::*;

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

#[cfg(feature = "with-shp")]
pub mod shp;

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

#[cfg(feature = "with-mlt")]
pub mod mlt;

pub mod bounds;

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
