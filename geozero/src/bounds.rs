use crate::{FeatureProcessor, GeomProcessor, PropertyProcessor};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Bounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Bounds {
    pub fn extend(&mut self, x: f64, y: f64) {
        if x < self.min_x {
            self.min_x = x;
        }
        if x > self.max_x {
            self.max_x = x;
        }
        if y < self.min_y {
            self.min_y = y;
        }
        if y > self.max_y {
            self.max_y = y;
        }
    }

    pub fn min_x(&self) -> f64 {
        self.min_x
    }

    pub fn min_y(&self) -> f64 {
        self.min_y
    }

    pub fn max_x(&self) -> f64 {
        self.max_x
    }

    pub fn max_y(&self) -> f64 {
        self.max_y
    }
}

/// Computes the bounds of a Geomtry
#[derive(Default, Debug)]
pub struct BoundsProcessor {
    bounds: Option<Bounds>,
}

impl BoundsProcessor {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn bounds(&self) -> Option<Bounds> {
        self.bounds.clone()
    }
}

impl GeomProcessor for BoundsProcessor {
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> crate::error::Result<()> {
        let Some(bounds) = self.bounds.as_mut() else {
            self.bounds = Some(Bounds {
                min_x: x,
                min_y: y,
                max_x: x,
                max_y: y,
            });
            return Ok(());
        };
        bounds.extend(x, y);
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
        idx: usize,
    ) -> crate::error::Result<()> {
        self.xy(x, y, idx)
    }
}

impl PropertyProcessor for BoundsProcessor {}
impl FeatureProcessor for BoundsProcessor {}

#[cfg(feature = "with-wkt")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::GeozeroGeometry;

    #[test]
    fn test_bounds() {
        let wkt = crate::wkt::Wkt("LINESTRING(1 2,3 4,5 6)");
        let mut bounds_processor = BoundsProcessor::new();

        wkt.process_geom(&mut bounds_processor).unwrap();

        assert_eq!(
            bounds_processor.bounds,
            Some(Bounds {
                min_x: 1.0,
                min_y: 2.0,
                max_x: 5.0,
                max_y: 6.0,
            })
        )
    }

    #[test]
    fn test_empty() {
        let wkt = crate::wkt::Wkt("LINESTRING EMPTY");
        let mut bounds_processor = BoundsProcessor::new();

        wkt.process_geom(&mut bounds_processor).unwrap();

        assert_eq!(bounds_processor.bounds, None);
    }
}
