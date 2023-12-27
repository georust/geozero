use crate::error::Result;
use crate::geometry_processor::GeomProcessor;
use crate::property_processor::PropertyProcessor;

/// Feature processing trait
#[allow(unused_variables)]
pub trait FeatureProcessor: GeomProcessor + PropertyProcessor {
    /// Begin of dataset processing
    ///
    /// ## Invariants
    ///
    /// - `dataset_begin` is called _only once_ for an entire dataset.
    /// - `dataset_begin` is called before all other methods, including `feature_begin`,
    ///   `properties_begin`, `geometry_begin`, and all methods from [`GeomProcessor`] and
    ///   [`PropertyProcessor`]
    fn dataset_begin(&mut self, name: Option<&str>) -> Result<()> {
        Ok(())
    }
    /// End of dataset processing
    ///
    /// ## Invariants
    ///
    /// - `dataset_end` is called _only once_ for an entire dataset.
    /// - No other methods may be called after `dataset_end`.
    fn dataset_end(&mut self) -> Result<()> {
        Ok(())
    }
    /// Begin of feature processing
    ///
    /// - `idx` refers to the positional row index in the dataset. For the `n`th row, `idx` will be
    ///   `n`.
    /// - `feature_begin` will be called before both `properties_begin` and `geometry_begin`.
    fn feature_begin(&mut self, idx: u64) -> Result<()> {
        Ok(())
    }
    /// End of feature processing
    ///
    /// - `idx` refers to the positional row index in the dataset. For the `n`th row, `idx` will be
    ///   `n`.
    /// - `feature_end` will be called after both `properties_end` and `geometry_end`.
    fn feature_end(&mut self, idx: u64) -> Result<()> {
        Ok(())
    }
    /// Begin of feature property processing
    ///
    /// ## Invariants
    ///
    /// - `properties_begin` will not be called a second time before `properties_end` is called.
    fn properties_begin(&mut self) -> Result<()> {
        Ok(())
    }
    /// End of feature property processing
    fn properties_end(&mut self) -> Result<()> {
        Ok(())
    }
    /// Begin of feature geometry processing
    ///
    /// ## Following events
    ///
    /// - Relevant methods from [`GeomProcessor`] will be called for each geometry.
    fn geometry_begin(&mut self) -> Result<()> {
        Ok(())
    }
    /// End of feature geometry processing
    fn geometry_end(&mut self) -> Result<()> {
        Ok(())
    }
}
