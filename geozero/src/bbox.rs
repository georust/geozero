use crate::error::Result;
use crate::GeomProcessor;

#[derive(Clone, PartialEq, Debug)]
/// Bounding Box
pub struct Bbox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Default for Bbox {
    fn default() -> Self {
        Bbox {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }
}

impl Bbox {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Bbox {
        Bbox {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn reset(&mut self) {
        self.min_x = f64::INFINITY;
        self.min_y = f64::INFINITY;
        self.max_x = f64::NEG_INFINITY;
        self.max_y = f64::NEG_INFINITY;
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn sum(mut a: Bbox, b: &Bbox) -> Bbox {
        a.expand(b);
        a
    }

    pub fn expand(&mut self, r: &Bbox) {
        if r.min_x < self.min_x {
            self.min_x = r.min_x;
        }
        if r.min_y < self.min_y {
            self.min_y = r.min_y;
        }
        if r.max_x > self.max_x {
            self.max_x = r.max_x;
        }
        if r.max_y > self.max_y {
            self.max_y = r.max_y;
        }
    }

    pub fn expand_xy(&mut self, x: f64, y: f64) {
        if x < self.min_x {
            self.min_x = x;
        }
        if y < self.min_y {
            self.min_y = y;
        }
        if x > self.max_x {
            self.max_x = x;
        }
        if y > self.max_y {
            self.max_y = y;
        }
    }

    pub fn intersects(&self, r: &Bbox) -> bool {
        if self.max_x < r.min_x {
            return false;
        }
        if self.max_y < r.min_y {
            return false;
        }
        if self.min_x > r.max_x {
            return false;
        }
        if self.min_y > r.max_y {
            return false;
        }
        true
    }
}

impl GeomProcessor for Bbox {
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
        self.expand_xy(x, y);
        Ok(())
    }
    fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        _z: Option<f64>,
        _m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
        _idx: usize,
    ) -> Result<()> {
        self.expand_xy(x, y);
        Ok(())
    }
    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        self.reset();
        Ok(())
    }
    fn multipoint_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.reset();
        Ok(())
    }
    fn linestring_begin(&mut self, tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        if tagged {
            self.reset();
        }
        Ok(())
    }
    fn multilinestring_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.reset();
        Ok(())
    }
    fn polygon_begin(&mut self, tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        if tagged {
            self.reset();
        }
        Ok(())
    }
    fn multipolygon_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.reset();
        Ok(())
    }
    fn geometrycollection_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.reset();
        Ok(())
    }
    fn circularstring_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.reset();
        Ok(())
    }
    fn compoundcurve_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.reset();
        Ok(())
    }
    fn curvepolygon_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.reset();
        Ok(())
    }
    fn multicurve_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.reset();
        Ok(())
    }
    fn multisurface_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.reset();
        Ok(())
    }
    fn triangle_begin(&mut self, tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        if tagged {
            self.reset();
        }
        Ok(())
    }
    fn polyhedralsurface_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.reset();
        Ok(())
    }
    fn tin_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.reset();
        Ok(())
    }
}
