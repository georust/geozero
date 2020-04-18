use crate::DebugReader;

pub struct Dimensions {
    /// height
    pub z: bool,
    /// measurement
    pub m: bool,
    /// geodetic decimal year time
    pub t: bool,
    /// time nanosecond measurement
    pub tm: bool,
}

pub trait GeomProcessor {
    /// Additional dimensions requested from reader
    fn dimensions(&self) -> Dimensions {
        Dimensions {
            z: false,
            m: false,
            t: false,
            tm: false,
        }
    }
    /// Point without additional dimensions
    fn pointxy(&mut self, _x: f64, _y: f64, _idx: usize) {}
    /// Point with additional dimensions
    fn point(
        &mut self,
        _x: f64,
        _y: f64,
        _z: Option<f64>,
        _m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
        _idx: usize,
    ) {
    }
    fn point_begin(&mut self, _idx: usize) {}
    fn point_end(&mut self) {}
    fn multipoint_begin(&mut self, _size: usize, _idx: usize) {}
    fn multipoint_end(&mut self) {}
    fn line_begin(&mut self, _size: usize, _idx: usize) {}
    fn line_end(&mut self, _idx: usize) {}
    fn multiline_begin(&mut self, _size: usize, _idx: usize) {}
    fn multiline_end(&mut self) {}
    fn ring_begin(&mut self, _size: usize, _idx: usize) {}
    fn ring_end(&mut self, _idx: usize) {}
    fn poly_begin(&mut self, _size: usize, _idx: usize) {}
    fn poly_end(&mut self, _idx: usize) {}
    fn subpoly_begin(&mut self, _size: usize, _idx: usize) {}
    fn subpoly_end(&mut self, _idx: usize) {}
    fn multipoly_begin(&mut self, _size: usize, _idx: usize) {}
    fn multipoly_end(&mut self) {}
}

impl GeomProcessor for DebugReader {
    fn pointxy(&mut self, x: f64, y: f64, _idx: usize) {
        print!("pointxy({} {}) ", x, y);
    }
    fn point_begin(&mut self, _idx: usize) {
        print!("point_begin ");
    }
    fn point_end(&mut self) {
        println!("point_end ");
    }
    fn multipoint_begin(&mut self, _size: usize, _idx: usize) {
        print!("multipoint_begin ");
    }
    fn multipoint_end(&mut self) {
        println!("multipoint_end ");
    }
    fn line_begin(&mut self, _size: usize, _idx: usize) {
        print!("line_begin ");
    }
    fn line_end(&mut self, _idx: usize) {
        println!("line_end ");
    }
    fn multiline_begin(&mut self, _size: usize, _idx: usize) {
        print!("multiline_begin ");
    }
    fn multiline_end(&mut self) {
        println!("multiline_end ");
    }
    fn ring_begin(&mut self, _size: usize, _idx: usize) {
        print!("ring_begin ");
    }
    fn ring_end(&mut self, _idx: usize) {
        println!("ring_end ");
    }
    fn poly_begin(&mut self, _size: usize, _idx: usize) {
        print!("poly_begin ");
    }
    fn poly_end(&mut self, _idx: usize) {
        println!("poly_end ");
    }
    fn subpoly_begin(&mut self, _size: usize, _idx: usize) {
        print!("subpoly_begin ");
    }
    fn subpoly_end(&mut self, _idx: usize) {
        println!("subpoly_end ");
    }
    fn multipoly_begin(&mut self, _size: usize, _idx: usize) {
        print!("multipoly_begin ");
    }
    fn multipoly_end(&mut self) {
        println!("multipoly_end ");
    }
}
