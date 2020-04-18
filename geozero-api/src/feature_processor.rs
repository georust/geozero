use crate::geometry_processor::GeomProcessor;
use crate::property_processor::PropertyProcessor;
use crate::DebugReader;

pub trait FeatureProcessor: GeomProcessor + PropertyProcessor {
    fn dataset_begin(&mut self, _name: Option<&str>) {}
    fn dataset_end(&mut self) {}
    fn feature_begin(&mut self, _idx: u64) {}
    fn feature_end(&mut self, _idx: u64) {}
    fn properties_begin(&mut self) {}
    fn properties_end(&mut self) {}
    fn geometry_begin(&mut self) {}
    fn geometry_end(&mut self) {}
}

impl FeatureProcessor for DebugReader {} //TODO
