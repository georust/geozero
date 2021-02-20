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
//! * [geozero-core](https://docs.rs/geozero-core)
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
#[cfg(feature = "core")]
mod formats;
mod geometry_processor;
mod multiplex;
mod property_processor;

pub use driver::*;
pub use feature_processor::*;
#[cfg(feature = "core")]
pub use formats::*;
pub use geometry_processor::*;
pub use multiplex::*;
pub use property_processor::*;

/// GeoJSON conversions.
#[cfg(feature = "core")]
pub mod geojson {
    pub use crate::formats::geojson_reader::*;
    pub use crate::formats::geojson_writer::*;
}
#[cfg(feature = "core")]
pub use crate::formats::geojson_writer::conversion::*;

/// [geo-types](https://github.com/georust/geo) conversions.
#[cfg(feature = "core")]
pub mod geo_types {
    pub use crate::formats::geo_types_reader::*;
    pub use crate::formats::geo_types_writer::*;
}
#[cfg(feature = "core")]
pub use crate::formats::geo_types_writer::conversion::*;

/// Well-Known Binary (WKB) conversions.
///
/// # Usage examples:
///
/// Convert a EWKB geometry to WKT:
///
/// ```
/// use geozero_core::{ToWkt, wkb::Ewkb};
///
/// let wkb = Ewkb(vec![1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 64, 0, 0, 0, 0, 0, 0, 52, 192]);
/// assert_eq!(wkb.to_wkt().unwrap(), "POINT(10 -20)");
/// ```
#[cfg(feature = "core")]
pub mod wkb {
    pub use crate::formats::wkb_common::*;
    pub use crate::formats::wkb_reader::*;
    pub use crate::formats::wkb_writer::*;
}
#[cfg(feature = "core")]
pub use crate::formats::wkb_writer::conversion::*;

/// Well-Known Text (WKT) conversions.
///
/// OpenGIS Simple Features Specification For SQL Revision 1.1, Chapter 3.2.5
#[cfg(feature = "core")]
pub mod wkt {
    pub use crate::formats::wkt_writer::*;
}
#[cfg(feature = "core")]
pub use crate::formats::wkt_writer::conversion::*;

/// [GEOS](https://github.com/georust/geos) conversions.
#[cfg(feature = "geos-lib")]
pub mod geos {
    pub use crate::formats::geos_reader::*;
    pub use crate::formats::geos_writer::*;
}
#[cfg(feature = "geos-lib")]
pub use crate::formats::geos_writer::conversion::*;

/// [GDAL](https://github.com/georust/gdal) conversions.
#[cfg(feature = "gdal-lib")]
pub mod gdal {
    pub use crate::formats::gdal_reader::*;
    pub use crate::formats::gdal_writer::*;
}
#[cfg(feature = "gdal-lib")]
pub use crate::formats::gdal_writer::conversion::*;

/// Geopackage geometry type encoding/decoding.
#[cfg(feature = "gpkg")]
pub mod gpkg {
    pub use crate::formats::geopackage::*;
}

#[cfg(feature = "core")]
pub use crate::formats::svg::conversion::*;

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
