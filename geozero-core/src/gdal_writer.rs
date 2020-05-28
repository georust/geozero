use gdal::vector::Geometry;
use gdal_sys::OGRwkbGeometryType;
use geozero::error::{GeozeroError, Result};
use geozero::{FeatureProcessor, GeomProcessor, PropertyProcessor};

/// Generator for [GEOS](https://github.com/georust/geos) geometry type
pub struct GdalWriter {
    geom: Geometry,
    // current line/ring of geom (non-owned)
    line: Geometry,
}

impl<'a> GdalWriter {
    pub fn new() -> Self {
        GdalWriter {
            geom: Geometry::empty(OGRwkbGeometryType::wkbPoint).unwrap(),
            line: Geometry::empty(OGRwkbGeometryType::wkbLineString).unwrap(),
        }
    }
    pub fn geometry(&self) -> &Geometry {
        &self.geom
    }
}

pub(crate) fn from_gdal_err(error: gdal::errors::Error) -> GeozeroError {
    GeozeroError::Geometry(error.to_string())
}

impl GeomProcessor for GdalWriter {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        match self.geom.geometry_type() {
            OGRwkbGeometryType::wkbPoint | OGRwkbGeometryType::wkbLineString => {
                self.geom.set_point_2d(idx, (x, y))
            }
            OGRwkbGeometryType::wkbMultiPoint => {
                let mut point =
                    Geometry::empty(OGRwkbGeometryType::wkbPoint).map_err(from_gdal_err)?;
                point.set_point_2d(0, (x, y));
                self.geom.add_geometry(point).map_err(from_gdal_err)?;
            }
            OGRwkbGeometryType::wkbMultiLineString
            | OGRwkbGeometryType::wkbPolygon
            | OGRwkbGeometryType::wkbMultiPolygon => {
                self.line.set_point_2d(idx, (x, y));
            }
            _ => {
                return Err(GeozeroError::Geometry(
                    "Unsupported geometry type".to_string(),
                ))
            }
        }
        Ok(())
    }
    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        self.geom = Geometry::empty(OGRwkbGeometryType::wkbPoint).map_err(from_gdal_err)?;
        Ok(())
    }
    fn multipoint_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.geom = Geometry::empty(OGRwkbGeometryType::wkbMultiPoint).map_err(from_gdal_err)?;
        Ok(())
    }
    fn linestring_begin(&mut self, tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        if tagged {
            self.geom =
                Geometry::empty(OGRwkbGeometryType::wkbLineString).map_err(from_gdal_err)?;
        } else {
            match self.geom.geometry_type() {
                OGRwkbGeometryType::wkbMultiLineString => {
                    let line = Geometry::empty(OGRwkbGeometryType::wkbLineString)
                        .map_err(from_gdal_err)?;
                    self.geom.add_geometry(line).map_err(from_gdal_err)?;

                    let n = self.geom.geometry_count();
                    self.line = unsafe { self.geom._get_geometry(n - 1) };
                }
                OGRwkbGeometryType::wkbPolygon => {
                    let ring = Geometry::empty(OGRwkbGeometryType::wkbLinearRing)
                        .map_err(from_gdal_err)?;
                    self.geom.add_geometry(ring).map_err(from_gdal_err)?;

                    let n = self.geom.geometry_count();
                    self.line = unsafe { self.geom._get_geometry(n - 1) };
                }
                OGRwkbGeometryType::wkbMultiPolygon => {
                    let ring = Geometry::empty(OGRwkbGeometryType::wkbLinearRing)
                        .map_err(from_gdal_err)?;
                    let n = self.geom.geometry_count();
                    let mut poly = unsafe { self.geom._get_geometry(n - 1) };
                    poly.add_geometry(ring).map_err(from_gdal_err)?;

                    let n = poly.geometry_count();
                    self.line = unsafe { poly._get_geometry(n - 1) };
                }
                _ => {
                    return Err(GeozeroError::Geometry(
                        "Unsupported geometry type".to_string(),
                    ))
                }
            };
        }
        Ok(())
    }
    fn multilinestring_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.geom =
            Geometry::empty(OGRwkbGeometryType::wkbMultiLineString).map_err(from_gdal_err)?;
        Ok(())
    }
    fn polygon_begin(&mut self, tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        let poly = Geometry::empty(OGRwkbGeometryType::wkbPolygon).map_err(from_gdal_err)?;
        if tagged {
            self.geom = poly;
        } else {
            self.geom.add_geometry(poly).map_err(from_gdal_err)?;
        }
        Ok(())
    }
    fn multipolygon_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.geom = Geometry::empty(OGRwkbGeometryType::wkbMultiPolygon).map_err(from_gdal_err)?;
        Ok(())
    }
}

impl PropertyProcessor for GdalWriter {}
impl FeatureProcessor for GdalWriter {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::geojson_reader::read_geojson;

    #[test]
    fn point_geom() {
        let geojson = r#"{"type": "Point", "coordinates": [1, 1]}"#;
        let wkt = "POINT (1 1)";
        let mut geom = GdalWriter::new();
        assert!(read_geojson(geojson.as_bytes(), &mut geom).is_ok());
        assert_eq!(geom.geometry().wkt().unwrap(), wkt);
    }

    #[test]
    fn multipoint_geom() {
        let geojson = r#"{"type": "MultiPoint", "coordinates": [[1, 1], [2, 2]]}"#;
        let wkt = "MULTIPOINT (1 1,2 2)";
        let mut geom = GdalWriter::new();
        assert!(read_geojson(geojson.as_bytes(), &mut geom).is_ok());
        assert_eq!(geom.geometry().wkt().unwrap(), wkt);
    }

    #[test]
    fn line_geom() {
        let geojson = r#"{"type": "LineString", "coordinates": [[1,1], [2,2]]}"#;
        let wkt = "LINESTRING (1 1,2 2)";
        let mut geom = GdalWriter::new();
        read_geojson(geojson.as_bytes(), &mut geom).unwrap();
        assert!(read_geojson(geojson.as_bytes(), &mut geom).is_ok());
        assert_eq!(geom.geometry().wkt().unwrap(), wkt);
    }

    // #[test]
    // fn line_geom_3d() {
    //     let geojson = r#"{"type": "LineString", "coordinates": [[1,1,10], [2,2,20]]}"#;
    //     let wkt = "LINESTRING (1 1 10, 2 2 20)";
    //     let mut geom = GdalWriter::new();
    //     assert!(read_geojson(geojson.as_bytes(), &mut geom).is_ok());
    //     assert_eq!(geom.geometry().wkt().unwrap(), wkt);
    // }

    #[test]
    fn multiline_geom() {
        let geojson =
            r#"{"type": "MultiLineString", "coordinates": [[[1,1],[2,2]],[[3,3],[4,4]]]}"#;
        let wkt = "MULTILINESTRING ((1 1,2 2),(3 3,4 4))";
        let mut geom = GdalWriter::new();
        assert!(read_geojson(geojson.as_bytes(), &mut geom).is_ok());
        assert_eq!(geom.geometry().wkt().unwrap(), wkt);
    }

    #[test]
    fn polygon_geom() {
        let geojson = r#"{"type": "Polygon", "coordinates": [[[0, 0], [0, 3], [3, 3], [3, 0], [0, 0]],[[0.2, 0.2], [0.2, 2], [2, 2], [2, 0.2], [0.2, 0.2]]]}"#;
        let wkt = "POLYGON ((0 0,0 3,3 3,3 0,0 0),(0.2 0.2,0.2 2.0,2 2,2.0 0.2,0.2 0.2))";
        let mut geom = GdalWriter::new();
        assert!(read_geojson(geojson.as_bytes(), &mut geom).is_ok());
        assert_eq!(geom.geometry().wkt().unwrap(), wkt);
    }

    #[test]
    fn multipolygon_geom() {
        let geojson =
            r#"{"type": "MultiPolygon", "coordinates": [[[[0,0],[0,1],[1,1],[1,0],[0,0]]]]}"#;
        let wkt = "MULTIPOLYGON (((0 0,0 1,1 1,1 0,0 0)))";
        let mut geom = GdalWriter::new();
        assert!(read_geojson(geojson.as_bytes(), &mut geom).is_ok());
        assert_eq!(geom.geometry().wkt().unwrap(), wkt);
    }

    // #[test]
    // fn geometry_collection_geom() {
    //     let geojson = r#"{"type": "Point", "coordinates": [1, 1]}"#;
    //     let wkt = "GEOMETRYCOLLECTION(POINT(1 1), LINESTRING(1 1, 2 2))";
    //     let mut geom = GdalWriter::new();
    //     assert!(read_geojson(geojson.as_bytes(), &mut geom).is_ok());
    //     assert_eq!(geom.geometry().wkt().unwrap(), wkt);
    // }

    #[test]
    fn gdal_error() {
        let mut geom = GdalWriter::new();
        assert!(geom.point_begin(0).is_ok());
        assert_eq!(
            geom.polygon_begin(false, 0, 0).err().unwrap().to_string(),
            "processing geometry `OGR method \'OGR_G_AddGeometryDirectly\' returned error: \'3\'`"
                .to_string()
        );
    }

    #[test]
    fn gdal_internal_error() {
        let mut geom = GdalWriter::new();
        assert!(geom.point_begin(0).is_ok());
        assert!(geom.xy(0.0, 0.0, 1).is_ok());
        // Writes "ERROR 6: Only i == 0 is supported" to stderr (see CPLSetErrorHandler)
    }
}
