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
//! geozero-api = "0.1"
//! ```
//!
//! ## Zero-copy geometry reader
//!
//! Geometries can be accessed by implementing the `GeomReader` trait.
//!
//! ```rust
//! use geozero_api::GeomReader;
//!
//! struct CoordPrinter;
//!
//! impl GeomReader for CoordPrinter {
//!     fn pointxy(&mut self, x: f64, y: f64, _idx: usize) {
//!         println!("({} {})", x, y);
//!     }
//! }
//! ```
//!
//! ## Zero-copy feature access
//!
//! Properties can be accessed by implementing the `PropertyReader` trait.
//!
//! ```rust
//! use geozero_api::{PropertyReader, ColumnValue};
//!
//! struct PropertyPrinter;
//!
//! impl PropertyReader for PropertyPrinter {
//!     fn property(&mut self, i: usize, n: &str, v: ColumnValue) -> bool {
//!         println!("columnidx: {} name: {} value: {:?}", i, n, v);
//!         false // don't abort
//!     }
//! }
//! ```

mod feature_reader;
mod geometry_reader;
mod property_reader;

pub use feature_reader::*;
pub use geometry_reader::*;
pub use property_reader::*;
