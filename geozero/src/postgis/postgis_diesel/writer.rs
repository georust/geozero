use std::io::Read;

use postgis_diesel::types::{
    AnyPoint, GeometryCollection, GeometryContainer, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, PointM, PointT, PointZ, PointZM, Polygon,
};

use crate::error::{GeozeroError, Result};
use crate::wkb::{self, FromWkb};
use crate::{CoordDimensions, GeomProcessor};

/// Generator for postgis_diesel geometry types.
pub struct PostgisDieselWriter<P: PointT> {
    geom: Option<GeometryContainer<P>>,
    srid: Option<u32>,
    // Enclosing geometry collections, innermost last
    collections: Vec<Vec<GeometryContainer<P>>>,
    // Polygons of a multi-polygon
    polygons: Option<Vec<Polygon<P>>>,
    // Rings of a polygon, or lines of a multi-linestring
    line_strings: Option<Vec<Vec<P>>>,
    // Coordinates of a point or line string
    coords: Option<Vec<P>>,
}

impl<P: PointT> Default for PostgisDieselWriter<P> {
    fn default() -> Self {
        Self {
            geom: None,
            srid: None,
            collections: Vec::new(),
            polygons: None,
            line_strings: None,
            coords: None,
        }
    }
}

impl<P: PointT> PostgisDieselWriter<P> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take_geometry(&mut self) -> Option<GeometryContainer<P>> {
        self.geom.take()
    }

    fn make_point(&self, x: f64, y: f64, z: Option<f64>, m: Option<f64>) -> Result<P> {
        P::new_point(x, y, self.srid, z, m).map_err(|e| GeozeroError::Geometry(e.to_string()))
    }

    fn push_coord(&mut self, point: P) -> Result<()> {
        let coords = self
            .coords
            .as_mut()
            .ok_or(GeozeroError::Geometry("Not ready for coords".to_string()))?;
        coords.push(point);
        Ok(())
    }

    fn take_coords(&mut self, geometry: &str) -> Result<Vec<P>> {
        self.coords
            .take()
            .ok_or(GeozeroError::Geometry(format!("No coords for {geometry}")))
    }

    /// Attaches a completed geometry to the enclosing collection, or to the output if there is none.
    fn finish_geometry(&mut self, geometry: GeometryContainer<P>) -> Result<()> {
        if let Some(collection) = self.collections.last_mut() {
            collection.push(geometry);
        } else {
            self.geom = Some(geometry);
        }
        Ok(())
    }
}

impl<P: PointT> GeomProcessor for PostgisDieselWriter<P> {
    fn dimensions(&self) -> CoordDimensions {
        CoordDimensions::xyzm()
    }

    fn srid(&mut self, srid: Option<i32>) -> Result<()> {
        self.srid = srid.map(|s| s as u32);
        Ok(())
    }

    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
        let point = self.make_point(x, y, None, None)?;
        self.push_coord(point)
    }

    fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
        _idx: usize,
    ) -> Result<()> {
        let point = self.make_point(x, y, z, m)?;
        self.push_coord(point)
    }

    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        self.coords = Some(Vec::with_capacity(1));
        Ok(())
    }

    fn point_end(&mut self, _idx: usize) -> Result<()> {
        let point = self
            .take_coords("Point")?
            .into_iter()
            .next()
            .ok_or(GeozeroError::Geometry("Empty point".to_string()))?;
        self.finish_geometry(GeometryContainer::Point(point))
    }

    fn multipoint_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.coords = Some(Vec::with_capacity(size));
        Ok(())
    }

    fn multipoint_end(&mut self, _idx: usize) -> Result<()> {
        let points = self.take_coords("MultiPoint")?;
        self.finish_geometry(GeometryContainer::MultiPoint(MultiPoint {
            points,
            srid: self.srid,
        }))
    }

    fn linestring_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        self.coords = Some(Vec::with_capacity(size));
        Ok(())
    }

    fn linestring_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        let points = self.take_coords("LineString")?;
        if tagged {
            return self.finish_geometry(GeometryContainer::LineString(LineString {
                points,
                srid: self.srid,
            }));
        }
        // Untagged: a ring of the enclosing polygon, or a line of the enclosing multi-linestring
        let line_strings = self.line_strings.as_mut().ok_or(GeozeroError::Geometry(
            "Missing container for LineString".to_string(),
        ))?;
        line_strings.push(points);
        Ok(())
    }

    fn multilinestring_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.line_strings = Some(Vec::with_capacity(size));
        Ok(())
    }

    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        let lines = self
            .line_strings
            .take()
            .ok_or(GeozeroError::Geometry(
                "No lines for MultiLineString".to_string(),
            ))?
            .into_iter()
            .map(|points| LineString {
                points,
                srid: self.srid,
            })
            .collect();
        self.finish_geometry(GeometryContainer::MultiLineString(MultiLineString {
            lines,
            srid: self.srid,
        }))
    }

    fn polygon_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        self.line_strings = Some(Vec::with_capacity(size));
        Ok(())
    }

    fn polygon_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        let rings = self.line_strings.take().ok_or(GeozeroError::Geometry(
            "Missing rings for Polygon".to_string(),
        ))?;
        let polygon = Polygon {
            rings,
            srid: self.srid,
        };
        if tagged {
            return self.finish_geometry(GeometryContainer::Polygon(polygon));
        }
        // Untagged: a member of the enclosing multi-polygon
        let polygons = self.polygons.as_mut().ok_or(GeozeroError::Geometry(
            "Missing container for Polygon".to_string(),
        ))?;
        polygons.push(polygon);
        Ok(())
    }

    fn multipolygon_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.polygons = Some(Vec::with_capacity(size));
        Ok(())
    }

    fn multipolygon_end(&mut self, _idx: usize) -> Result<()> {
        let polygons = self.polygons.take().ok_or(GeozeroError::Geometry(
            "Missing polygons for MultiPolygon".to_string(),
        ))?;
        self.finish_geometry(GeometryContainer::MultiPolygon(MultiPolygon {
            polygons,
            srid: self.srid,
        }))
    }

    fn geometrycollection_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.collections.push(Vec::with_capacity(size));
        Ok(())
    }

    fn geometrycollection_end(&mut self, _idx: usize) -> Result<()> {
        let geometries = self.collections.pop().ok_or(GeozeroError::Geometry(
            "Unexpected geometry type".to_string(),
        ))?;
        self.finish_geometry(GeometryContainer::GeometryCollection(GeometryCollection {
            geometries,
            srid: self.srid,
        }))
    }
}

macro_rules! impl_from_wkb {
    ($($t:ty),+) => {
        $(
            impl FromWkb for GeometryContainer<$t> {
                fn from_wkb<R: Read>(rdr: &mut R, dialect: wkb::WkbDialect) -> Result<Self> {
                    let mut writer = PostgisDieselWriter::<$t>::new();
                    wkb::process_wkb_type_geom(rdr, &mut writer, dialect)?;
                    writer
                        .take_geometry()
                        .ok_or(GeozeroError::Geometry("Missing geometry".to_string()))
                }
            }
        )+
    };
}

impl_from_wkb!(Point, PointZ, PointM, PointZM, AnyPoint);
