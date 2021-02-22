//! API traits for reading and writing spatial datasets.
//!
//! Data readers can be categorized based on different criteria.
//!
//! Access to raw record data:
//! * Full (e.g. FlatGeobuf reader)
//! * Partial (e.g. DBF access in Shapefile reader)
//! * No access (e.g. WKB reader)
//!
//! Reader granularity:
//! * Record access (e.g. FlatGeobuf)
//! * Full dataset (e.g. GeoJSON)
//!
//! Features are usually consumed by datasource iterators.
//! The current feature can be processed with `FeatureAccess` processing API methods.
//! Some datasources process features during consumation (e.g. reading from file).

use crate::error::Result;
use crate::feature_processor::FeatureProcessor;
use crate::property_processor::{
    PropertyProcessor, PropertyReadType, PropertyReader, PropertyReaderIdx,
};
use crate::{CoordDimensions, GeomProcessor};
use async_trait::async_trait;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::Path;

/// Geometry processing trait.
pub trait GeozeroGeometry {
    /// Process geometry.
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()>
    where
        Self: Sized;
    /// Empty geometry.
    fn empty() -> Self
    where
        Self: Sized;
    /// Dimensions of geometry
    fn dims(&self) -> CoordDimensions {
        CoordDimensions::xy()
    }
    /// SRID of geometry
    fn srid(&self) -> Option<i32> {
        None
    }
}

/// Geometry reader trait.
pub trait GeozeroGeometryReader {
    fn read_geom<R: Read, P: GeomProcessor>(reader: R, processor: &mut P) -> Result<()>;
}

// Datasource feature consumer trait.
pub trait GeozeroDatasource {
    /// Consume and process all selected features.
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()>;
}

// --- *Draft* datasource reader + writer API ---

pub struct OpenOpts {} //TBD: read_only, etc.

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Extent {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

// How can this made extensible (e.g. property filter)? Maybe with a builder pattern?
pub struct SelectOpts {
    pub extent: Option<Extent>,
}

// We need a combined trait to allow storing the trait object in a struct
pub trait ReadSeek: Read + Seek {}

// Implement it for common Read + Seek inputs
impl<R: Read + Seek> ReadSeek for std::io::BufReader<R> {}
impl<R: Read + Seek> ReadSeek for seek_bufread::BufReader<R> {}
impl ReadSeek for std::fs::File {}
impl<R: AsRef<[u8]>> ReadSeek for std::io::Cursor<R> {}

/// Datasource reader API (*Experimental*)
pub trait Reader<'a> {
    fn open<R: 'a + ReadSeek>(reader: &'a mut R, opts: &OpenOpts) -> Result<Self>
    where
        Self: Sized;
    fn select(&mut self, opts: &SelectOpts) -> Result<()>;
    /// Consume and process all selected features
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()>;
    //TODO: Add iterator function (-> &dyn Feature) ?
    //TODO: Add feature by id access function (-> &dyn Feature) ?
}

#[async_trait]
/// Async datasource HTTP reader API (*Experimental*)
pub trait HttpReader {
    async fn open(url: String, opts: &OpenOpts) -> Result<Self>
    where
        Self: Sized;
    async fn select(&mut self, opts: &SelectOpts) -> Result<()>;
    /// Read and process all selected features
    async fn process<P: FeatureProcessor + Send>(&mut self, processor: &mut P) -> Result<()>;
}

pub struct CreateOpts {} //TBD: read_only, etc.

/// Datasource writer API (*Experimental*)
pub trait Writer {
    fn open<P: AsRef<Path>>(path: P, opts: &CreateOpts) -> Result<Self>
    where
        Self: Sized;
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<Self>
    where
        Self: Sized;
}

/// Feature processing API
pub trait FeatureAccess: FeatureProperties + GeozeroGeometry {
    /// Process feature geometries and properties.
    fn process<P: FeatureProcessor>(&self, processor: &mut P, idx: u64) -> Result<()>
    where
        Self: Sized,
    {
        processor.feature_begin(idx)?;
        processor.properties_begin()?;
        let _ = self.process_properties(processor)?;
        processor.properties_end()?;
        processor.geometry_begin()?;
        self.process_geom(processor)?;
        processor.geometry_end()?;
        processor.feature_end(idx)
    }
}

/// Feature properties processing API
pub trait FeatureProperties {
    /// Process feature properties.
    fn process_properties<P: PropertyProcessor>(&self, processor: &mut P) -> Result<bool>;
    /// Get property value by name
    fn property<T: PropertyReadType>(&self, name: &str) -> Option<T> {
        let mut reader = PropertyReader { name, value: None };
        if self.process_properties(&mut reader).is_ok() {
            reader.value
        } else {
            None
        }
    }
    /// Get property value by number
    fn property_n<T: PropertyReadType>(&self, n: usize) -> Option<T> {
        let mut reader = PropertyReaderIdx {
            idx: n,
            value: None,
        };
        if self.process_properties(&mut reader).is_ok() {
            reader.value
        } else {
            None
        }
    }
    /// Return all properties in a HashMap
    /// Use `process_properties` for zero-copy access
    fn properties(&self) -> Result<HashMap<String, String>> {
        let mut properties = HashMap::new();
        let _ = self.process_properties(&mut properties)?;
        Ok(properties)
    }
}
