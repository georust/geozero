use crate::error::{GeozeroError, Result};
use crate::{FeatureProcessor, GeomProcessor, PropertyProcessor};
use geos::{CoordDimensions, CoordSeq, GResult, Geometry as GGeometry};

/// Generator for GEOS geometry type.
pub struct GeosWriter<'a> {
    pub(crate) geom: GGeometry<'a>,
    // CoordSeq for Points, Lines and Rings
    cs: Vec<CoordSeq<'a>>,
    // Polygons or MultiPolygons
    polys: Vec<GGeometry<'a>>,
}

impl<'a> GeosWriter<'a> {
    pub fn new() -> Self {
        GeosWriter {
            geom: GGeometry::create_empty_point().unwrap(),
            cs: Vec::new(),
            polys: Vec::new(),
        }
    }
    fn add_coord_seq(&mut self, len: usize) -> Result<()> {
        self.cs
            .push(CoordSeq::new(len as u32, CoordDimensions::TwoD)?);
        Ok(())
    }
    pub fn geometry(&self) -> &GGeometry<'a> {
        &self.geom
    }
}

impl GeomProcessor for GeosWriter<'_> {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        if self.cs.is_empty() {
            return Err(GeozeroError::Geometry("CoordSeq missing".to_string()));
        }
        let n = self.cs.len() - 1;
        let coord_seq = &mut self.cs[n];
        coord_seq.set_x(idx, x)?;
        coord_seq.set_y(idx, y)?;
        Ok(())
    }
    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        self.cs = Vec::with_capacity(1);
        self.add_coord_seq(1)?;
        Ok(())
    }
    fn point_end(&mut self, _idx: usize) -> Result<()> {
        let cs = self
            .cs
            .pop()
            .ok_or_else(|| GeozeroError::Geometry("CoordSeq missing".to_string()))?;
        self.geom = GGeometry::create_point(cs)?;
        Ok(())
    }
    fn multipoint_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.cs = Vec::with_capacity(1);
        self.add_coord_seq(size)?;
        Ok(())
    }
    fn multipoint_end(&mut self, _idx: usize) -> Result<()> {
        // Create points from CoordSeq elements
        let cs = self
            .cs
            .pop()
            .ok_or_else(|| GeozeroError::Geometry("CoordSeq missing".to_string()))?;
        let size = cs.size()?;
        let ggpts = (0..size)
            .map(|i| {
                GGeometry::create_point(
                    CoordSeq::new_from_vec(&[&[cs.get_x(i).unwrap(), cs.get_y(i).unwrap()]])
                        .unwrap(),
                )
            })
            .collect::<GResult<Vec<GGeometry>>>()?;
        self.geom = GGeometry::create_multipoint(ggpts)?;
        Ok(())
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, _idx: usize) -> Result<()> {
        if tagged {
            self.cs = Vec::with_capacity(1);
        } // else allocated in multilinestring_begin or polygon_begin
        self.add_coord_seq(size)?;
        Ok(())
    }
    fn linestring_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        if tagged {
            let cs = self
                .cs
                .pop()
                .ok_or_else(|| GeozeroError::Geometry("CoordSeq missing".to_string()))?;
            self.geom = GGeometry::create_line_string(cs)?;
        }
        Ok(())
    }
    fn multilinestring_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.cs = Vec::with_capacity(size);
        Ok(())
    }
    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        let gglines = self
            .cs
            .drain(..)
            .map(GGeometry::create_line_string)
            .collect::<GResult<Vec<GGeometry>>>()?;
        self.geom = GGeometry::create_multiline_string(gglines)?;
        Ok(())
    }
    fn polygon_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        self.cs = Vec::with_capacity(size);
        Ok(())
    }
    fn polygon_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        if self.cs.is_empty() {
            return Err(GeozeroError::Geometry("CoordSeq missing".to_string()));
        }
        // TODO: We need to ensure that rings of polygons are closed
        // to create valid GEOS LinearRings
        let exterior_ring = GGeometry::create_linear_ring(self.cs.remove(0))?;
        let interiors = self
            .cs
            .drain(..)
            .map(GGeometry::create_linear_ring)
            .collect::<GResult<Vec<GGeometry>>>()?;
        let gpoly = GGeometry::create_polygon(exterior_ring, interiors)?;
        if tagged {
            self.geom = gpoly;
        } else {
            self.polys.push(gpoly)
        }
        Ok(())
    }
    fn multipolygon_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.polys = Vec::with_capacity(size);
        Ok(())
    }
    fn multipolygon_end(&mut self, _idx: usize) -> Result<()> {
        self.geom = GGeometry::create_multipolygon(std::mem::take(&mut self.polys))?;
        Ok(())
    }
}

impl PropertyProcessor for GeosWriter<'_> {}
impl FeatureProcessor for GeosWriter<'_> {}

#[cfg(test)]
#[cfg(feature = "with-geojson")]
mod test {
    use super::*;
    use crate::geojson::{read_geojson, GeoJson};
    use crate::ToGeos;
    use geos::Geom;
    use std::convert::TryFrom;

    #[test]
    fn point_geom() {
        let geojson = r#"{"type": "Point", "coordinates": [1, 1]}"#;
        let wkt = "POINT (1.0000000000000000 1.0000000000000000)";
        let mut geos = GeosWriter::new();
        assert!(read_geojson(geojson.as_bytes(), &mut geos).is_ok());
        assert_eq!(geos.geometry().to_wkt().unwrap(), wkt);
    }

    #[test]
    fn multipoint_geom() {
        let geojson = GeoJson(r#"{"type": "MultiPoint", "coordinates": [[1, 1], [2, 2]]}"#);
        let wkt = "MULTIPOINT (1.0000000000000000 1.0000000000000000, 2.0000000000000000 2.0000000000000000)";
        let geos = geojson.to_geos().unwrap();
        assert_eq!(geos.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn line_geom() {
        let geojson = GeoJson(r#"{"type": "LineString", "coordinates": [[1,1], [2,2]]}"#);
        let wkt = "LINESTRING (1.0000000000000000 1.0000000000000000, 2.0000000000000000 2.0000000000000000)";
        let geos = geojson.to_geos().unwrap();
        assert_eq!(geos.to_wkt().unwrap(), wkt);
    }

    // #[test]
    // fn line_geom_3d() {
    //     let geojson = GeoJson(r#"{"type": "LineString", "coordinates": [[1,1,10], [2,2,20]]}"#);
    //     let wkt = "LINESTRING (1 1 10, 2 2 20)";
    //     let geos = geojson.to_geos().unwrap();
    //     assert_eq!(geos.to_wkt().unwrap(), wkt);
    // }

    #[test]
    fn multiline_geom() {
        let geojson =
            GeoJson(r#"{"type": "MultiLineString", "coordinates": [[[1,1],[2,2]],[[3,3],[4,4]]]}"#);
        let wkt = "MULTILINESTRING ((1.0000000000000000 1.0000000000000000, 2.0000000000000000 2.0000000000000000), (3.0000000000000000 3.0000000000000000, 4.0000000000000000 4.0000000000000000))";
        let geos = geojson.to_geos().unwrap();
        assert_eq!(geos.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn polygon_geom() {
        let geojson = GeoJson(
            r#"{"type": "Polygon", "coordinates": [[[0, 0], [0, 3], [3, 3], [3, 0], [0, 0]],[[0.2, 0.2], [0.2, 2], [2, 2], [2, 0.2], [0.2, 0.2]]]}"#,
        );
        let wkt = "POLYGON ((0.0000000000000000 0.0000000000000000, 0.0000000000000000 3.0000000000000000, 3.0000000000000000 3.0000000000000000, 3.0000000000000000 0.0000000000000000, 0.0000000000000000 0.0000000000000000), (0.2000000000000000 0.2000000000000000, 0.2000000000000000 2.0000000000000000, 2.0000000000000000 2.0000000000000000, 2.0000000000000000 0.2000000000000000, 0.2000000000000000 0.2000000000000000))";
        let geos = geojson.to_geos().unwrap();
        assert_eq!(geos.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn multipolygon_geom() {
        let geojson = GeoJson(
            r#"{"type": "MultiPolygon", "coordinates": [[[[0,0],[0,1],[1,1],[1,0],[0,0]]]]}"#,
        );
        let wkt = "MULTIPOLYGON (((0.0000000000000000 0.0000000000000000, 0.0000000000000000 1.0000000000000000, 1.0000000000000000 1.0000000000000000, 1.0000000000000000 0.0000000000000000, 0.0000000000000000 0.0000000000000000)))";
        let geos = geojson.to_geos().unwrap();
        assert_eq!(geos.to_wkt().unwrap(), wkt);
    }

    // #[test]
    // fn geometry_collection_geom() {
    //     let geojson = GeoJson(r#"{"type": "Point", "coordinates": [1, 1]}"#);
    //     let wkt = "GEOMETRYCOLLECTION(POINT(1 1), LINESTRING(1 1, 2 2))";
    //     let geos = geojson.to_geos().unwrap();
    //     assert_eq!(geos.to_wkt().unwrap(), wkt);
    // }

    #[test]
    #[cfg(feature = "with-geo")]
    fn geo_to_geos() -> Result<()> {
        use std::str::FromStr;
        let geo =
            geo_types::Geometry::try_from(wkt::Wkt::from_str("POINT (10 20)").unwrap()).unwrap();
        let geos = geo.to_geos()?;
        assert_eq!(
            &geos.to_wkt().unwrap(),
            "POINT (10.0000000000000000 20.0000000000000000)"
        );
        Ok(())
    }
}
