use geozero::{FeatureProcessor, GeomProcessor, PropertyProcessor};
use std::io::Write;

/// WKT according to OpenGIS Simple Features Specification For SQL Revision 1.1
pub struct WktWriter<'a, W: Write> {
    out: &'a mut W,
}

impl<'a, W: Write> WktWriter<'a, W> {
    pub fn new(out: &'a mut W) -> WktWriter<'a, W> {
        WktWriter { out }
    }
}

impl<W: Write> GeomProcessor for WktWriter<'_, W> {
    fn xy(&mut self, x: f64, y: f64, idx: usize) {
        if idx == 0 {
            self.out.write(&format!("{} {}", x, y).as_bytes()).unwrap();
        } else {
            self.out
                .write(&format!(", {} {}", x, y).as_bytes())
                .unwrap();
        }
    }
    fn point_begin(&mut self, _idx: usize) {
        self.out.write(b"POINT (").unwrap();
    }
    fn point_end(&mut self, _idx: usize) {
        self.out.write(b")").unwrap();
    }
    fn multipoint_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(b"MULTIPOINT (").unwrap();
    }
    fn multipoint_end(&mut self, _idx: usize) {
        self.out.write(b")").unwrap();
    }
    fn linestring_begin(&mut self, tagged: bool, _size: usize, idx: usize) {
        if tagged {
            self.out.write(b"LINESTRING (").unwrap();
        } else {
            if idx == 0 {
                self.out.write(b"(").unwrap();
            } else {
                self.out.write(b", (").unwrap();
            }
        }
    }
    fn linestring_end(&mut self, _tagged: bool, _idx: usize) {
        self.out.write(b")").unwrap();
    }
    fn multilinestring_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(b"MULTILINESTRING (").unwrap();
    }
    fn multilinestring_end(&mut self, _idx: usize) {
        self.out.write(b")").unwrap();
    }
    fn polygon_begin(&mut self, tagged: bool, _size: usize, idx: usize) {
        if tagged {
            self.out.write(b"POLYGON (").unwrap();
        } else {
            if idx == 0 {
                self.out.write(b"(").unwrap();
            } else {
                self.out.write(b", (").unwrap();
            }
        }
    }
    fn polygon_end(&mut self, _tagged: bool, _idx: usize) {
        self.out.write(b")").unwrap();
    }
    fn multipolygon_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(b"MULTIPOLYGON (").unwrap();
    }
    fn multipolygon_end(&mut self, _idx: usize) {
        self.out.write(b")").unwrap();
    }
    // GEOMETRYCOLLECTION (POINT (10 10),
    // POINT (30 30),
    // LINESTRING (15 15, 20 20))’
}

impl<W: Write> PropertyProcessor for WktWriter<'_, W> {}

impl<W: Write> FeatureProcessor for WktWriter<'_, W> {}
