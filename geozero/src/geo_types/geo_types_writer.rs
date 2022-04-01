use crate::error::{GeozeroError, Result};
use crate::{FeatureProcessor, GeomProcessor, PropertyProcessor};
use geo_types::*;
use std::mem;

/// Generator for geo-types geometry type.
pub struct GeoWriter {
    pub(crate) geom: Geometry<f64>,
    // Polygon rings or MultiLineString members
    line_strings: Vec<LineString<f64>>,
}

impl GeoWriter {
    pub fn new() -> GeoWriter {
        GeoWriter {
            geom: Point::new(0., 0.).into(),
            line_strings: Vec::new(),
        }
    }
    pub fn geometry(&self) -> &Geometry<f64> {
        &self.geom
    }
}

impl GeomProcessor for GeoWriter {
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
        if self.line_strings.len() > 0 {
            let idx = self.line_strings.len() - 1;
            self.line_strings[idx].0.push(Coordinate { x, y });
        } else {
            match &mut self.geom {
                Geometry::Point(_) => {
                    self.geom = Point::new(x, y).into();
                }
                Geometry::MultiPoint(mp) => {
                    mp.0.push(Point::new(x, y));
                }
                _ => {
                    return Err(GeozeroError::Geometry(
                        "Unexpected geometry type".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        self.geom = Point::new(0., 0.).into();
        Ok(())
    }
    fn multipoint_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.geom = MultiPoint(Vec::<Point<f64>>::with_capacity(size)).into();
        Ok(())
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, _idx: usize) -> Result<()> {
        let line_string = LineString(Vec::<Coordinate<f64>>::with_capacity(size));
        if tagged {
            self.line_strings = Vec::with_capacity(1);
        } // else allocated in multilinestring_begin or polygon_begin
        self.line_strings.push(line_string);
        Ok(())
    }
    fn linestring_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        if tagged {
            self.geom = self
                .line_strings
                .pop()
                .ok_or(GeozeroError::Geometry("LineString missing".to_string()))?
                .into();
        }
        Ok(())
    }
    fn multilinestring_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.line_strings = Vec::with_capacity(size);
        Ok(())
    }
    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        self.geom = MultiLineString(mem::take(&mut self.line_strings)).into();
        Ok(())
    }
    fn polygon_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        self.line_strings = Vec::with_capacity(size);
        Ok(())
    }
    fn polygon_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        if self.line_strings.len() == 0 {
            // Instead of erroring, we could write a Polygon whose exterior is an empty LineString.
            return Err(GeozeroError::Geometry("Missing LineString".to_string()));
        }
        let exterior = self.line_strings.remove(0);
        let polygon = Polygon::new(exterior, mem::take(&mut self.line_strings));
        if tagged {
            self.geom = polygon.into();
        } else if let Geometry::MultiPolygon(mp) = &mut self.geom {
            mp.0.push(polygon);
        } else {
            return Err(GeozeroError::Geometry(
                "Unexpected geometry type".to_string(),
            ));
        }
        Ok(())
    }
    fn multipolygon_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.geom = MultiPolygon(Vec::<Polygon<f64>>::with_capacity(size)).into();
        Ok(())
    }
}

impl PropertyProcessor for GeoWriter {}

impl FeatureProcessor for GeoWriter {}

#[cfg(test)]
#[cfg(feature = "with-geojson")]
mod test {
    use super::*;
    use crate::geojson::{read_geojson, GeoJson};
    use crate::ToGeo;
    use geo::algorithm::coords_iter::CoordsIter;

    #[test]
    fn line_string() -> Result<()> {
        let geojson = r#"{"type": "LineString", "coordinates": [[1875038.447610231,-3269648.6879248763],[1874359.641504197,-3270196.812984864],[1874141.0428635243,-3270953.7840121365],[1874440.1778162003,-3271619.4315206874],[1876396.0598222911,-3274138.747656357],[1876442.0805243007,-3275052.60551469],[1874739.312657555,-3275457.333765534]]}"#;
        let mut geo = GeoWriter::new();
        assert!(read_geojson(geojson.as_bytes(), &mut geo).is_ok());
        println!("{:?}", geo.geometry());
        match geo.geometry() {
            Geometry::LineString(line) => {
                assert_eq!(line.coords_count(), 7);
                assert_eq!(
                    line.points().next().unwrap(),
                    Point::new(1875038.447610231, -3269648.6879248763)
                );
            }
            _ => assert!(false),
        }
        Ok(())
    }

    #[test]
    fn multipolygon() -> Result<()> {
        let geojson = GeoJson(
            r#"{"type": "MultiPolygon", "coordinates": [[[[173.020375,-40.919052],[173.247234,-41.331999],[173.958405,-40.926701],[174.247587,-41.349155],[174.248517,-41.770008],[173.876447,-42.233184],[173.22274,-42.970038],[172.711246,-43.372288],[173.080113,-43.853344],[172.308584,-43.865694],[171.452925,-44.242519],[171.185138,-44.897104],[170.616697,-45.908929],[169.831422,-46.355775],[169.332331,-46.641235],[168.411354,-46.619945],[167.763745,-46.290197],[166.676886,-46.219917],[166.509144,-45.852705],[167.046424,-45.110941],[168.303763,-44.123973],[168.949409,-43.935819],[169.667815,-43.555326],[170.52492,-43.031688],[171.12509,-42.512754],[171.569714,-41.767424],[171.948709,-41.514417],[172.097227,-40.956104],[172.79858,-40.493962],[173.020375,-40.919052]]],[[[174.612009,-36.156397],[175.336616,-37.209098],[175.357596,-36.526194],[175.808887,-36.798942],[175.95849,-37.555382],[176.763195,-37.881253],[177.438813,-37.961248],[178.010354,-37.579825],[178.517094,-37.695373],[178.274731,-38.582813],[177.97046,-39.166343],[177.206993,-39.145776],[176.939981,-39.449736],[177.032946,-39.879943],[176.885824,-40.065978],[176.508017,-40.604808],[176.01244,-41.289624],[175.239567,-41.688308],[175.067898,-41.425895],[174.650973,-41.281821],[175.22763,-40.459236],[174.900157,-39.908933],[173.824047,-39.508854],[173.852262,-39.146602],[174.574802,-38.797683],[174.743474,-38.027808],[174.697017,-37.381129],[174.292028,-36.711092],[174.319004,-36.534824],[173.840997,-36.121981],[173.054171,-35.237125],[172.636005,-34.529107],[173.007042,-34.450662],[173.551298,-35.006183],[174.32939,-35.265496],[174.612009,-36.156397]]]]}"#,
        );
        let geo = geojson.to_geo().unwrap();
        println!("{:?}", geo);
        match geo {
            Geometry::MultiPolygon(mp) => {
                let poly = mp.clone().into_iter().next().unwrap();
                assert_eq!(
                    poly.exterior().points().next().unwrap(),
                    Point::new(173.020375, -40.919052)
                );
            }
            _ => assert!(false),
        }
        Ok(())
    }

    #[test]
    fn to_geo() -> Result<()> {
        let geom: geo_types::Geometry<f64> = geo_types::Point::new(10.0, 20.0).into();
        assert_eq!(geom.clone().to_geo().unwrap(), geom);
        Ok(())
    }
}
