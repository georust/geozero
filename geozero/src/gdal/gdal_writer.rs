use crate::error::{GeozeroError, Result};
use crate::gdal::gdal_error::GdalError;
use crate::{CoordDimensions, FeatureProcessor, GeomProcessor, PropertyProcessor};

use gdal::vector::Geometry;
use gdal_sys::OGRwkbGeometryType;

/// Generator for GDAL geometry type.
pub struct GdalWriter {
    dims: CoordDimensions,
    pub(crate) geom: Geometry,
    // current line/ring of geom (non-owned)
    line: Geometry,
}

impl GdalWriter {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_dims(dims: CoordDimensions) -> Self {
        GdalWriter {
            dims,
            ..Self::default()
        }
    }
    pub fn geometry(&self) -> &Geometry {
        &self.geom
    }
    fn wkb_type(&mut self, base: OGRwkbGeometryType::Type) -> OGRwkbGeometryType::Type {
        let mut type_id = base;
        if self.dims.z {
            type_id += 1000;
        }
        if self.dims.m {
            type_id += 2000;
        }
        type_id
    }
    fn empty_geom(&mut self, base: OGRwkbGeometryType::Type) -> Result<Geometry> {
        Geometry::empty(self.wkb_type(base)).map_err(|e| e.into())
    }
}

impl Default for GdalWriter {
    fn default() -> Self {
        GdalWriter {
            dims: CoordDimensions::default(),
            geom: Geometry::empty(OGRwkbGeometryType::wkbPoint).unwrap(),
            line: Geometry::empty(OGRwkbGeometryType::wkbLineString).unwrap(),
        }
    }
}

fn wkb_base_type(wkb_type: OGRwkbGeometryType::Type) -> OGRwkbGeometryType::Type {
    wkb_type % 1000
}

impl From<gdal::errors::GdalError> for GeozeroError {
    fn from(error: gdal::errors::GdalError) -> Self {
        GeozeroError::Geometry(error.to_string())
    }
}

impl GeomProcessor for GdalWriter {
    fn dimensions(&self) -> CoordDimensions {
        self.dims
    }
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        match self.geom.geometry_type() {
            OGRwkbGeometryType::wkbPoint | OGRwkbGeometryType::wkbLineString => {
                self.geom.set_point_2d(idx, (x, y));
            }
            OGRwkbGeometryType::wkbMultiPoint => {
                let mut point = self.empty_geom(OGRwkbGeometryType::wkbPoint)?;
                point.set_point_2d(0, (x, y));
                self.geom.add_geometry(point)?;
            }
            OGRwkbGeometryType::wkbMultiLineString
            | OGRwkbGeometryType::wkbPolygon
            | OGRwkbGeometryType::wkbMultiPolygon => {
                self.line.set_point_2d(idx, (x, y));
            }
            _ => {
                let unsupported_type = self.geom.geometry_type();
                return Err(GdalError::UnsupportedGeometryType(unsupported_type))?;
            }
        }
        Ok(())
    }
    fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        _m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
        idx: usize,
    ) -> Result<()> {
        let z = z.unwrap_or(0.0);
        match wkb_base_type(self.geom.geometry_type()) {
            OGRwkbGeometryType::wkbPoint | OGRwkbGeometryType::wkbLineString => {
                self.geom.set_point(idx, (x, y, z));
            }
            OGRwkbGeometryType::wkbMultiPoint => {
                let mut point = self.empty_geom(OGRwkbGeometryType::wkbPoint)?;
                point.set_point(0, (x, y, z));
                self.geom.add_geometry(point)?;
            }
            OGRwkbGeometryType::wkbMultiLineString
            | OGRwkbGeometryType::wkbPolygon
            | OGRwkbGeometryType::wkbMultiPolygon => {
                self.line.set_point(idx, (x, y, z));
            }
            _ => {
                let unsupported_type = self.geom.geometry_type();
                return Err(GdalError::UnsupportedGeometryType(unsupported_type))?;
            }
        }
        Ok(())
    }
    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        self.geom = self.empty_geom(OGRwkbGeometryType::wkbPoint)?;
        Ok(())
    }
    fn multipoint_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.geom = self.empty_geom(OGRwkbGeometryType::wkbMultiPoint)?;
        Ok(())
    }
    fn linestring_begin(&mut self, tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        if tagged {
            self.geom = self.empty_geom(OGRwkbGeometryType::wkbLineString)?;
        } else {
            match wkb_base_type(self.geom.geometry_type()) {
                OGRwkbGeometryType::wkbMultiLineString => {
                    let line = self.empty_geom(OGRwkbGeometryType::wkbLineString)?;
                    self.geom.add_geometry(line)?;

                    let n = self.geom.geometry_count();
                    self.line = unsafe { self.geom.get_unowned_geometry(n - 1) };
                }
                OGRwkbGeometryType::wkbPolygon => {
                    let ring = self.empty_geom(OGRwkbGeometryType::wkbLinearRing)?;
                    self.geom.add_geometry(ring)?;

                    let n = self.geom.geometry_count();
                    self.line = unsafe { self.geom.get_unowned_geometry(n - 1) };
                }
                OGRwkbGeometryType::wkbMultiPolygon => {
                    let ring = self.empty_geom(OGRwkbGeometryType::wkbLinearRing)?;
                    let n = self.geom.geometry_count();
                    let mut poly = unsafe { self.geom.get_unowned_geometry(n - 1) };
                    poly.add_geometry(ring)?;

                    let n = poly.geometry_count();
                    self.line = unsafe { poly.get_unowned_geometry(n - 1) };
                }
                unsupported_type => {
                    return Err(GdalError::UnsupportedGeometryType(unsupported_type))?;
                }
            };
        }
        Ok(())
    }
    fn multilinestring_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.geom = self.empty_geom(OGRwkbGeometryType::wkbMultiLineString)?;
        Ok(())
    }
    fn polygon_begin(&mut self, tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        let poly = self.empty_geom(OGRwkbGeometryType::wkbPolygon)?;
        if tagged {
            self.geom = poly;
        } else {
            self.geom.add_geometry(poly)?;
        }
        Ok(())
    }
    fn multipolygon_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.geom = self.empty_geom(OGRwkbGeometryType::wkbMultiPolygon)?;
        Ok(())
    }
}

impl PropertyProcessor for GdalWriter {}
impl FeatureProcessor for GdalWriter {}

#[cfg(test)]
#[cfg(all(feature = "with-wkt", feature = "with-geojson"))]
mod test {
    use super::*;
    use crate::geojson::{read_geojson, GeoJson};
    use crate::ToGdal;

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
        let geojson = GeoJson(r#"{"type": "MultiPoint", "coordinates": [[1, 1], [2, 2]]}"#);
        let wkt = "MULTIPOINT (1 1,2 2)";
        let geom = geojson.to_gdal().unwrap();
        assert_eq!(geom.wkt().unwrap(), wkt);
    }

    #[test]
    fn line_geom() {
        let geojson = GeoJson(r#"{"type": "LineString", "coordinates": [[1,1], [2,2]]}"#);
        let wkt = "LINESTRING (1 1,2 2)";
        let geom = geojson.to_gdal().unwrap();
        assert_eq!(geom.wkt().unwrap(), wkt);
    }

    // TODO: 3D output is broken!
    // #[test]
    // fn line_geom_3d() {
    //     let wkt = "LINESTRING (1 1 10, 2 2 20)";
    //     let gdal = Geometry::from_wkt(wkt).unwrap();
    //     let geom = gdal
    //         .to_gdal_ndim(CoordDimensions {
    //             z: true,
    //             m: false,
    //             t: false,
    //             tm: false,
    //         })
    //         .unwrap();
    //     assert_eq!(geom.wkt().unwrap(), wkt);
    // }

    #[test]
    fn multiline_geom() {
        let geojson =
            GeoJson(r#"{"type": "MultiLineString", "coordinates": [[[1,1],[2,2]],[[3,3],[4,4]]]}"#);
        let wkt = "MULTILINESTRING ((1 1,2 2),(3 3,4 4))";
        let geom = geojson.to_gdal().unwrap();
        assert_eq!(geom.wkt().unwrap(), wkt);
    }

    #[test]
    fn polygon_geom() {
        let geojson = r#"{
            "type": "Polygon",
            "coordinates": [[
                [0, 0], [0, 3], [3, 3], [3, 0], [0, 0]
            ],[
                [0.2, 0.2], [0.2, 2], [2, 2], [2, 0.2], [0.2, 0.2]
            ]]
        }"#;
        let geojson = GeoJson(geojson);
        let wkt = "POLYGON ((0 0,0 3,3 3,3 0,0 0),(0.2 0.2,0.2 2.0,2 2,2.0 0.2,0.2 0.2))";
        let geom = geojson.to_gdal().unwrap();
        assert_eq!(geom.wkt().unwrap(), wkt);
    }

    #[test]
    fn multipolygon_geom() {
        let geojson = GeoJson(
            r#"{"type": "MultiPolygon", "coordinates": [[[[0,0],[0,1],[1,1],[1,0],[0,0]]]]}"#,
        );
        let wkt = "MULTIPOLYGON (((0 0,0 1,1 1,1 0,0 0)))";
        let geom = geojson.to_gdal().unwrap();
        assert_eq!(geom.wkt().unwrap(), wkt);
    }

    // #[test]
    // fn geometry_collection_geom() {
    //     let geojson = GeoJson(r#"{"type": "Point", "coordinates": [1, 1]}"#);
    //     let wkt = "GEOMETRYCOLLECTION(POINT(1 1), LINESTRING(1 1, 2 2))";
    //     let geom = geojson.to_gdal().unwrap();
    //     assert_eq!(geom.wkt().unwrap(), wkt);
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
