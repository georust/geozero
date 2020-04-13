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
    fn pointxy(&mut self, x: f64, y: f64, idx: usize) {
        self.p1.pointxy(x, y, idx);
        self.p2.pointxy(x, y, idx);
    }
    fn point(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        m: Option<f64>,
        t: Option<f64>,
        tm: Option<u64>,
        idx: usize,
    ) {
        self.p1.point(x, y, z, m, t, tm, idx);
        self.p2.point(x, y, z, m, t, tm, idx);
    }
    fn point_begin(&mut self, idx: usize) {
        self.p1.point_begin(idx);
        self.p2.point_begin(idx);
    }
    fn point_end(&mut self) {
        self.p1.point_end();
        self.p2.point_end();
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) {
        self.p1.multipoint_begin(size, idx);
        self.p2.multipoint_begin(size, idx);
    }
    fn multipoint_end(&mut self) {
        self.p1.multipoint_end();
        self.p2.multipoint_end();
    }
    fn line_begin(&mut self, size: usize, idx: usize) {
        self.p1.line_begin(size, idx);
        self.p2.line_begin(size, idx);
    }
    fn line_end(&mut self, _idx: usize) {
        self.p1.line_end(_idx);
        self.p2.line_end(_idx);
    }
    fn multiline_begin(&mut self, size: usize, idx: usize) {
        self.p1.multiline_begin(size, idx);
        self.p2.multiline_begin(size, idx);
    }
    fn multiline_end(&mut self) {
        self.p1.multiline_end();
        self.p2.multiline_end();
    }
    fn ring_begin(&mut self, size: usize, idx: usize) {
        self.p1.ring_begin(size, idx);
        self.p2.ring_begin(size, idx);
    }
    fn ring_end(&mut self, _idx: usize) {
        self.p1.ring_end(_idx);
        self.p2.ring_end(_idx);
    }
    fn poly_begin(&mut self, size: usize, idx: usize) {
        self.p1.poly_begin(size, idx);
        self.p2.poly_begin(size, idx);
    }
    fn poly_end(&mut self, _idx: usize) {
        self.p1.poly_end(_idx);
        self.p2.poly_end(_idx);
    }
    fn subpoly_begin(&mut self, size: usize, idx: usize) {
        self.p1.subpoly_begin(size, idx);
        self.p2.subpoly_begin(size, idx);
    }
    fn subpoly_end(&mut self, _idx: usize) {
        self.p1.subpoly_end(_idx);
        self.p2.subpoly_end(_idx);
    }
    fn multipoly_begin(&mut self, size: usize, idx: usize) {
        self.p1.multipoly_begin(size, idx);
        self.p2.multipoly_begin(size, idx);
    }
    fn multipoly_end(&mut self) {
        self.p1.multipoly_end();
        self.p2.multipoly_end();
    }
}

impl<P1: FeatureProcessor, P2: FeatureProcessor> PropertyProcessor for Multiplexer<P1, P2> {
    fn property(&mut self, i: usize, colname: &str, colval: &ColumnValue) -> bool {
        self.p1.property(i, colname, colval) || self.p2.property(i, colname, colval)
    }
}
