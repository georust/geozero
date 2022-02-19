use crate::error::{GeozeroError, Result};
use crate::{FeatureProcessor, GeomProcessor, PropertyProcessor};
use std::mem;

/// Generator for MVT geometry type.
pub struct MvtWriter {
    pub(crate) geom: Geometry<f64>,
    // Polygon rings or MultiLineString members
    line_strings: Vec<LineString<f64>>,
}

impl MvtWriter {
    pub fn new() -> MvtWriter {
        MvtWriter {
            geom: Point::new(0., 0.).into(),
            line_strings: Vec::new(),
        }
    }
    pub fn geometry(&self) -> &Geometry<f64> {
        &self.geom
    }
}

impl GeomProcessor for MvtWriter {
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

impl PropertyProcessor for MvtWriter {}

impl FeatureProcessor for MvtWriter {}

#[cfg(test)]
#[cfg(feature = "with-geojson")]
mod test {
    use super::*;
    use crate::geojson::{read_geojson, GeoJson};
    use crate::ToMvt;
    use mvt::Geom;
    use std::convert::TryFrom;

    #[test]
    fn point_geom() {
        let geojson = r#"{"type": "Point", "coordinates": [1, 1]}"#;
        let wkt = "POINT (1.0000000000000000 1.0000000000000000)";
        let mut mvt = MvtWriter::new();
        assert!(read_geojson(geojson.as_bytes(), &mut mvt).is_ok());
        assert_eq!(mvt.geometry().to_wkt().unwrap(), wkt);
    }

    #[test]
    fn multipoint_geom() {
        let geojson = GeoJson(r#"{"type": "MultiPoint", "coordinates": [[1, 1], [2, 2]]}"#);
        let wkt = "MULTIPOINT (1.0000000000000000 1.0000000000000000, 2.0000000000000000 2.0000000000000000)";
        let mvt = geojson.to_mvt().unwrap();
        assert_eq!(mvt.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn line_geom() {
        let geojson = GeoJson(r#"{"type": "LineString", "coordinates": [[1,1], [2,2]]}"#);
        let wkt = "LINESTRING (1.0000000000000000 1.0000000000000000, 2.0000000000000000 2.0000000000000000)";
        let mvt = geojson.to_mvt().unwrap();
        assert_eq!(mvt.to_wkt().unwrap(), wkt);
    }

    // #[test]
    // fn line_geom_3d() {
    //     let geojson = GeoJson(r#"{"type": "LineString", "coordinates": [[1,1,10], [2,2,20]]}"#);
    //     let wkt = "LINESTRING (1 1 10, 2 2 20)";
    //     let mvt = geojson.to_mvt().unwrap();
    //     assert_eq!(mvt.to_wkt().unwrap(), wkt);
    // }

    #[test]
    fn multiline_geom() {
        let geojson =
            GeoJson(r#"{"type": "MultiLineString", "coordinates": [[[1,1],[2,2]],[[3,3],[4,4]]]}"#);
        let wkt = "MULTILINESTRING ((1.0000000000000000 1.0000000000000000, 2.0000000000000000 2.0000000000000000), (3.0000000000000000 3.0000000000000000, 4.0000000000000000 4.0000000000000000))";
        let mvt = geojson.to_mvt().unwrap();
        assert_eq!(mvt.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn polygon_geom() {
        let geojson = GeoJson(
            r#"{"type": "Polygon", "coordinates": [[[0, 0], [0, 3], [3, 3], [3, 0], [0, 0]],[[0.2, 0.2], [0.2, 2], [2, 2], [2, 0.2], [0.2, 0.2]]]}"#,
        );
        let wkt = "POLYGON ((0.0000000000000000 0.0000000000000000, 0.0000000000000000 3.0000000000000000, 3.0000000000000000 3.0000000000000000, 3.0000000000000000 0.0000000000000000, 0.0000000000000000 0.0000000000000000), (0.2000000000000000 0.2000000000000000, 0.2000000000000000 2.0000000000000000, 2.0000000000000000 2.0000000000000000, 2.0000000000000000 0.2000000000000000, 0.2000000000000000 0.2000000000000000))";
        let mvt = geojson.to_mvt().unwrap();
        assert_eq!(mvt.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn multipolygon_geom() {
        let geojson = GeoJson(
            r#"{"type": "MultiPolygon", "coordinates": [[[[0,0],[0,1],[1,1],[1,0],[0,0]]]]}"#,
        );
        let wkt = "MULTIPOLYGON (((0.0000000000000000 0.0000000000000000, 0.0000000000000000 1.0000000000000000, 1.0000000000000000 1.0000000000000000, 1.0000000000000000 0.0000000000000000, 0.0000000000000000 0.0000000000000000)))";
        let mvt = geojson.to_mvt().unwrap();
        assert_eq!(mvt.to_wkt().unwrap(), wkt);
    }

    // #[test]
    // fn geometry_collection_geom() {
    //     let geojson = GeoJson(r#"{"type": "Point", "coordinates": [1, 1]}"#);
    //     let wkt = "GEOMETRYCOLLECTION(POINT(1 1), LINESTRING(1 1, 2 2))";
    //     let mvt = geojson.to_mvt().unwrap();
    //     assert_eq!(mvt.to_wkt().unwrap(), wkt);
    // }

    #[test]
    #[cfg(feature = "with-geo")]
    fn geo_to_mvt() -> Result<()> {
        let geo =
            geo_types::Geometry::try_from(wkt::Wkt::from_str("POINT (10 20)").unwrap()).unwrap();
        let mvt = geo.to_mvt()?;
        assert_eq!(
            &mvt.to_wkt().unwrap(),
            "POINT (10.0000000000000000 20.0000000000000000)"
        );
        Ok(())
    }
}
