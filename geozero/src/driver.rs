use crate::error::Result;
use crate::feature_processor::FeatureProcessor;
use async_trait::async_trait;
use std::io::{Read, Seek};
use std::path::Path;

pub trait Driver {}

pub struct OpenOpts {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Extent {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

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

pub trait Reader<'a> {
    fn open<R: 'a + ReadSeek>(reader: &'a mut R, _opts: &OpenOpts) -> Result<Self>
    where
        Self: Sized;
    fn select(&mut self, opts: &SelectOpts) -> Result<()>;
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()>;
}

#[async_trait]
pub trait HttpReader {
    async fn open(url: String, opts: &OpenOpts) -> Result<Self>
    where
        Self: Sized;
    async fn select(&mut self, opts: &SelectOpts) -> Result<()>;
    async fn process<P: FeatureProcessor + Send>(&mut self, processor: &mut P) -> Result<()>;
}

pub struct CreateOpts {}

pub trait Writer {
    fn open<P: AsRef<Path>>(path: P, opts: &CreateOpts) -> Result<Self>
    where
        Self: Sized;
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<Self>
    where
        Self: Sized;
}
