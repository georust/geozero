use crate::error::Result;
use crate::events::Event::*;
use crate::events::{Event, GeomEventProcessor, GeometryType};

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

impl GeomEventProcessor for Bbox {
    fn event(&mut self, event: Event, geom_type: GeometryType, collection: bool) -> Result<()> {
        match event {
            Xy(x, y, _idx) => {
                self.expand_xy(x, y);
            }
            Coordinate(x, y, _z, _m, _t, _tm, _idx) => {
                self.expand_xy(x, y);
            }
            PointBegin(_idifx) if !collection => {
                self.reset();
            }
            MultiPointBegin(_size, _idx) if !collection => {
                self.reset();
            }
            LineStringBegin(_size, _idx) if !collection => {
                if geom_type == GeometryType::LineString {
                    self.reset();
                }
            }
            MultiLineStringBegin(_size, _idx) if !collection => {
                self.reset();
            }
            PolygonBegin(_size, _idx) if !collection => {
                if geom_type == GeometryType::Polygon {
                    self.reset();
                }
            }
            MultiPolygonBegin(_size, _idx) if !collection => {
                self.reset();
            }
            GeometryCollectionBegin(_size, _idx) if !collection => {
                self.reset();
            }
            CircularStringBegin(_size, _idx) if !collection => {
                self.reset();
            }
            CompoundCurveBegin(_size, _idx) if !collection => {
                self.reset();
            }
            CurvePolygonBegin(_size, _idx) if !collection => {
                self.reset();
            }
            MultiCurveBegin(_size, _idx) if !collection => {
                self.reset();
            }
            MultiSurfaceBegin(_size, _idx) if !collection => {
                self.reset();
            }
            TriangleBegin(_size, _idx) if !collection => {
                if geom_type == GeometryType::Triangle {
                    self.reset();
                }
            }
            PolyhedralSurfaceBegin(_size, _idx) if !collection => {
                self.reset();
            }
            TinBegin(_sizeif, _idx) if !collection => {
                self.reset();
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "with-geojson")]
mod test {
    use super::*;
    use crate::api::GeozeroGeometry;
    use crate::events::GeomVisitor;
    use crate::geojson::GeoJson;

    #[test]
    fn polygon() -> Result<()> {
        let geojson = GeoJson(
            r#"{"type": "Polygon", "coordinates": [[[20.590247,41.855404],[20.463175,41.515089],[20.605182,41.086226],[21.02004,40.842727],[20.99999,40.580004],[20.674997,40.435],[20.615,40.110007],[20.150016,39.624998],[19.98,39.694993],[19.960002,39.915006],[19.406082,40.250773],[19.319059,40.72723],[19.40355,41.409566],[19.540027,41.719986],[19.371769,41.877548],[19.304486,42.195745],[19.738051,42.688247],[19.801613,42.500093],[20.0707,42.58863],[20.283755,42.32026],[20.52295,42.21787],[20.590247,41.855404]]]}"#,
        );
        let mut processor = Bbox::default();
        geojson.process_geom(&mut GeomVisitor::new(&mut processor))?;
        assert_eq!(
            processor,
            Bbox::new(19.304486, 39.624998, 21.02004, 42.688247)
        );
        Ok(())
    }

    #[test]
    fn geomcollection() -> Result<()> {
        let geojson = GeoJson(
            r#"{"type": "GeometryCollection", "geometries": [{"type": "Point", "coordinates": [100.1,0.1]},{"type": "LineString", "coordinates": [[101.1,0.1],[102.1,1.1]]}]}"#,
        );
        let mut processor = Bbox::default();
        geojson.process_geom(&mut GeomVisitor::new(&mut processor))?;
        assert_eq!(processor, Bbox::new(100.1, 0.1, 102.1, 1.1));
        Ok(())
    }
}
