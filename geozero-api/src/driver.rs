use crate::feature_processor::FeatureProcessor;
use crate::geometry_processor::GeomProcessor;
use crate::property_processor::PropertyProcessor;
use async_trait::async_trait;
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

pub trait Reader {
    fn open<P: AsRef<Path>>(path: P, opts: &OpenOpts) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn select(&mut self, opts: &SelectOpts);
    fn process<P>(&mut self, processor: &mut P)
    where
        P: FeatureProcessor + GeomProcessor + PropertyProcessor;
}

#[async_trait]
pub trait HttpReader {
    async fn open(url: String, opts: &OpenOpts) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    async fn select(&mut self, opts: &SelectOpts);
    async fn process<P>(&mut self, processor: &mut P)
    where
        P: FeatureProcessor + GeomProcessor + PropertyProcessor + Send;
}

pub struct CreateOpts {}

pub trait Writer {
    fn open<P: AsRef<Path>>(path: P, opts: &CreateOpts) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn process<P>(&mut self, processor: &mut P)
    where
        P: FeatureProcessor + GeomProcessor + PropertyProcessor;
}
