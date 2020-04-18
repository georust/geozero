use crate::feature_processor::FeatureProcessor;
use crate::geometry_processor::GeomProcessor;
use crate::property_processor::{ColumnValue, PropertyProcessor};

pub struct Multiplexer<P1: FeatureProcessor, P2: FeatureProcessor> {
    p1: P1,
    p2: P2,
}

impl<P1: FeatureProcessor, P2: FeatureProcessor> Multiplexer<P1, P2> {
    pub fn new(p1: P1, p2: P2) -> Multiplexer<P1, P2> {
        Multiplexer { p1, p2 }
    }
}

impl<P1: FeatureProcessor, P2: FeatureProcessor> FeatureProcessor for Multiplexer<P1, P2> {
    fn dataset_begin(&mut self, name: Option<&str>) {
        self.p1.dataset_begin(name);
        self.p2.dataset_begin(name);
    }
    fn dataset_end(&mut self) {
        self.p1.dataset_end();
        self.p2.dataset_end();
    }
    fn feature_begin(&mut self, idx: u64) {
        self.p1.feature_begin(idx);
        self.p2.feature_begin(idx);
    }
    fn feature_end(&mut self, idx: u64) {
        self.p1.feature_end(idx);
        self.p2.feature_end(idx);
    }
    fn properties_begin(&mut self) {
        self.p1.properties_begin();
        self.p2.properties_begin();
    }
    fn properties_end(&mut self) {
        self.p1.properties_end();
        self.p2.properties_end();
    }
    fn geometry_begin(&mut self) {
        self.p1.geometry_begin();
        self.p2.geometry_begin();
    }
    fn geometry_end(&mut self) {
        self.p1.geometry_end();
        self.p2.geometry_end();
    }
}

impl<P1: FeatureProcessor, P2: FeatureProcessor> GeomProcessor for Multiplexer<P1, P2> {
    fn xy(&mut self, x: f64, y: f64, idx: usize) {
        self.p1.xy(x, y, idx);
        self.p2.xy(x, y, idx);
    }
    fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        m: Option<f64>,
        t: Option<f64>,
        tm: Option<u64>,
        idx: usize,
    ) {
        self.p1.coordinate(x, y, z, m, t, tm, idx);
        self.p2.coordinate(x, y, z, m, t, tm, idx);
    }
    fn point_begin(&mut self, idx: usize) {
        self.p1.point_begin(idx);
        self.p2.point_begin(idx);
    }
    fn point_end(&mut self, idx: usize) {
        self.p1.point_end(idx);
        self.p2.point_end(idx);
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) {
        self.p1.multipoint_begin(size, idx);
        self.p2.multipoint_begin(size, idx);
    }
    fn multipoint_end(&mut self, idx: usize) {
        self.p1.multipoint_end(idx);
        self.p2.multipoint_end(idx);
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) {
        self.p1.linestring_begin(tagged, size, idx);
        self.p2.linestring_begin(tagged, size, idx);
    }
    fn linestring_end(&mut self, tagged: bool, idx: usize) {
        self.p1.linestring_end(tagged, idx);
        self.p2.linestring_end(tagged, idx);
    }
    fn multilinestring_begin(&mut self, size: usize, idx: usize) {
        self.p1.multilinestring_begin(size, idx);
        self.p2.multilinestring_begin(size, idx);
    }
    fn multilinestring_end(&mut self, idx: usize) {
        self.p1.multilinestring_end(idx);
        self.p2.multilinestring_end(idx);
    }
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) {
        self.p1.polygon_begin(tagged, size, idx);
        self.p2.polygon_begin(tagged, size, idx);
    }
    fn polygon_end(&mut self, tagged: bool, idx: usize) {
        self.p1.polygon_end(tagged, idx);
        self.p2.polygon_end(tagged, idx);
    }
    fn multipolygon_begin(&mut self, size: usize, idx: usize) {
        self.p1.multipolygon_begin(size, idx);
        self.p2.multipolygon_begin(size, idx);
    }
    fn multipolygon_end(&mut self, idx: usize) {
        self.p1.multipolygon_end(idx);
        self.p2.multipolygon_end(idx);
    }
}

impl<P1: FeatureProcessor, P2: FeatureProcessor> PropertyProcessor for Multiplexer<P1, P2> {
    fn property(&mut self, i: usize, colname: &str, colval: &ColumnValue) -> bool {
        self.p1.property(i, colname, colval) || self.p2.property(i, colname, colval)
    }
}
