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

use crate::error::Result;
use crate::feature_processor::FeatureProcessor;
use crate::geometry_processor::GeomProcessor;
use async_trait::async_trait;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::Path;

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
//TODO: move optional functions into separate traits
#[allow(unused_variables)]
pub trait Feature {
    /// Raw geometry access.
    fn geometry<T>(&self) -> Option<&T> {
        unimplemented!()
    }
    /// Consume and process feature.
    // Remark: HTTP reader may require Send
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        unimplemented!()
    }
    /// Consume and process geometry.
    fn process_geom<P: GeomProcessor>(&mut self, processor: &mut P) -> Result<()>;
    /// Return all properties in a HashMap.
    fn properties(&self) -> Result<HashMap<String, String>> {
        unimplemented!()
    }
}
