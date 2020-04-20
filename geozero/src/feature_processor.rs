use crate::error::Result;
use crate::geometry_processor::GeomProcessor;
use crate::property_processor::PropertyProcessor;

/// Feature processing trait
#[allow(unused_variables)]
pub trait FeatureProcessor: GeomProcessor + PropertyProcessor {
    /// Begin of dataset processing
    fn dataset_begin(&mut self, name: Option<&str>) -> Result<()> {
        Ok(())
    }
    /// End of dataset processing
    fn dataset_end(&mut self) -> Result<()> {
        Ok(())
    }
    /// Begin of feature processing
    fn feature_begin(&mut self, idx: u64) -> Result<()> {
        Ok(())
    }
    /// End of feature processing
    fn feature_end(&mut self, idx: u64) -> Result<()> {
        Ok(())
    }
    /// Begin of feature property processing
    fn properties_begin(&mut self) -> Result<()> {
        Ok(())
    }
    /// End of feature property processing
    fn properties_end(&mut self) -> Result<()> {
        Ok(())
    }
    /// Begin of feature geometry processing
    fn geometry_begin(&mut self) -> Result<()> {
        Ok(())
    }
    /// End of feature geometry processing
    fn geometry_end(&mut self) -> Result<()> {
        Ok(())
    }
}
