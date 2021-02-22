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
//!
//! ## Zero-copy geometry reader
//!
//! Geometries can be accessed by implementing the `GeomProcessor` trait.
//!
//! ```rust
//! use geozero::{GeomProcessor, error::Result};
//!
//! struct CoordPrinter;
//!
//! impl GeomProcessor for CoordPrinter {
//!     fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
//!         Ok(println!("({} {})", x, y))
//!     }
//! }
//! ```
//!
//! ## Zero-copy feature access
//!
//! Properties can be accessed by implementing the `PropertyProcessor` trait.
//!
//! ```rust
//! use geozero::{PropertyProcessor, ColumnValue, error::Result};
//!
//! struct PropertyPrinter;
//!
//! impl PropertyProcessor for PropertyPrinter {
//!     fn property(&mut self, i: usize, n: &str, v: &ColumnValue) -> Result<bool> {
//!         println!("columnidx: {} name: {} value: {:?}", i, n, v);
//!         Ok(false) // don't abort
//!     }
//! }
//! ```

mod driver;
pub mod error;
mod feature_processor;
mod geometry_processor;
mod multiplex;
mod property_processor;

pub use driver::*;
pub use feature_processor::*;
pub use geometry_processor::*;
pub use multiplex::*;
pub use property_processor::*;

#[cfg(feature = "gdal")]
pub mod gdal;
#[cfg(feature = "gdal")]
pub use crate::gdal::conversion::*;

#[cfg(feature = "core")]
pub mod geo_types;
#[cfg(feature = "core")]
pub use crate::geo_types::conversion::*;

#[cfg(feature = "core")]
pub mod geojson;
#[cfg(feature = "core")]
pub use crate::geojson::conversion::*;

#[cfg(feature = "geos-lib")]
pub mod geos;
#[cfg(feature = "geos-lib")]
pub use crate::geos::conversion::*;

#[cfg(feature = "gpkg")]
pub mod gpkg;

#[cfg(any(feature = "postgis-postgres", feature = "postgis-sqlx"))]
pub mod postgis;

#[cfg(feature = "core")]
pub mod svg;
#[cfg(feature = "core")]
pub use crate::svg::conversion::*;

#[cfg(feature = "tesselator")]
pub mod tessellator;

#[cfg(feature = "core")]
pub mod wkb;
#[cfg(feature = "core")]
pub use crate::wkb::conversion::*;

#[cfg(feature = "core")]
pub mod wkt;
#[cfg(feature = "core")]
pub use crate::wkt::conversion::*;

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
