use crate::shp::shp_reader::{NO_DATA, is_no_data};
use std::fmt;

/// Point with `x`, `y`, `m`, `z`
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct PointZ {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub m: f64,
}

impl PointZ {
    pub fn new(x: f64, y: f64, z: f64, m: f64) -> Self {
        Self { x, y, z, m }
    }
    pub fn x(&self) -> f64 {
        self.x
    }
    pub fn y(&self) -> f64 {
        self.y
    }
    pub fn z(&self) -> f64 {
        self.z
    }
    fn m(&self) -> f64 {
        self.m
    }
}

impl Default for PointZ {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            m: NO_DATA,
        }
    }
}

impl fmt::Display for PointZ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if is_no_data(self.m) {
            write!(
                f,
                "Point(x: {}, y: {}, z: {}, m: NO_DATA)",
                self.x, self.y, self.z
            )
        } else {
            write!(
                f,
                "Point(x: {}, y: {}, z: {}, m: {})",
                self.x, self.y, self.z, self.m
            )
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct BBoxZ {
    pub max: PointZ,
    pub min: PointZ,
}

impl BBoxZ {
    pub fn x_range(&self) -> [f64; 2] {
        [self.min.x(), self.max.x()]
    }

    pub fn y_range(&self) -> [f64; 2] {
        [self.min.y(), self.max.y()]
    }

    pub fn z_range(&self) -> [f64; 2] {
        [self.min.z(), self.max.z()]
    }

    pub fn m_range(&self) -> [f64; 2] {
        [self.min.m(), self.max.m()]
    }
}
