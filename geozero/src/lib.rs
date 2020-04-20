//! Zero-Copy reading and writing of geospatial data.
//!
//! GeoZero defines an API for reading geospatial data formats without an intermediate representation.
//! It defines traits which can be implemented to read and convert to an arbitrary format
//! or render geometries directly.
//!
//! Supported geometry types:
//! * [OGC Simple Features](https://en.wikipedia.org/wiki/Simple_Features)
//!
//! Supported dimensions: X, Y, Z, M, T
//!
//! ## Installation
//!
//! ```ini
//! [dependencies]
//! geozero = "0.2"
//! ```
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
