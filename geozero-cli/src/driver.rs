use async_trait::async_trait;
use geozero::error::Result;
use geozero::FeatureProcessor;
use std::io::{Read, Seek};

// --- Datasource reader + writer API ---

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

/// Datasource reader API
pub trait Reader<'a, R: Read + Seek> {
    fn open(reader: &'a mut R, opts: &OpenOpts) -> Result<Self>
    where
        Self: Sized;
    fn select(&mut self, opts: &SelectOpts) -> Result<()>;
    /// Consume and process all selected features
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()>;
}

#[async_trait]
/// Async datasource HTTP reader API
pub trait HttpReader {
    async fn open(url: String, opts: &OpenOpts) -> Result<Self>
    where
        Self: Sized;
    async fn select(&mut self, opts: &SelectOpts) -> Result<()>;
    /// Read and process all selected features
    async fn process<P: FeatureProcessor + Send>(&mut self, processor: &mut P) -> Result<()>;
}

// pub struct CreateOpts {} //TBD: read_only, etc.

// /// Datasource writer API
// pub trait Writer {
//     fn open<P: AsRef<Path>>(path: P, opts: &CreateOpts) -> Result<Self>
//     where
//         Self: Sized;
//     fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<Self>
//     where
//         Self: Sized;
// }

// --- FlatGeoBuf implementation ---

use flatgeobuf::FgbReader;

impl<'a, R: Read + Seek> Reader<'a, R> for FgbReader<'a, R> {
    fn open(reader: &'a mut R, _opts: &OpenOpts) -> Result<Self> {
        Ok(FgbReader::open(reader)?)
    }

    fn select(&mut self, opts: &SelectOpts) -> Result<()> {
        if let Some(bbox) = &opts.extent {
            self.select_bbox(bbox.minx, bbox.miny, bbox.maxx, bbox.maxy)?;
        } else {
            self.select_all()?;
        }
        Ok(())
    }

    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        self.process_features(processor)
    }
}

pub(crate) mod http {
    use crate::driver::{HttpReader, OpenOpts, SelectOpts};
    use async_trait::async_trait;
    use flatgeobuf::HttpFgbReader;
    use geozero::error::Result;
    use geozero::FeatureProcessor;

    #[async_trait]
    impl HttpReader for HttpFgbReader {
        async fn open(url: String, _opts: &OpenOpts) -> Result<Self> {
            Ok(HttpFgbReader::open(&url).await?)
        }

        async fn select(&mut self, opts: &SelectOpts) -> Result<()> {
            if let Some(bbox) = &opts.extent {
                self.select_bbox(bbox.minx, bbox.miny, bbox.maxx, bbox.maxy)
                    .await?;
            } else {
                self.select_all().await?;
            }
            Ok(())
        }

        async fn process<P: FeatureProcessor + Send>(&mut self, processor: &mut P) -> Result<()> {
            self.process_features(processor).await?;
            Ok(())
        }
    }
}
