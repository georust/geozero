use crate::error::{GeozeroError, Result};
use crate::wkb::{self, Ewkb, FromWkb};
use crate::{CoordDimensions, GeomProcessor, GeozeroGeometry};

use diesel::deserialize::{self, FromSql};
use diesel::pg::{self, Pg};
use diesel::serialize::{self, IsNull, Output, ToSql};
use postgis_diesel::types::{
    AnyPoint, GeometryCollection, GeometryContainer, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, PointM, PointT, PointZ, PointZM, Polygon,
};
use std::io::{Read, Write as _};

// ---------------------------------------------------------------------------
// SQL types (kept for backwards compatibility)
// ---------------------------------------------------------------------------

pub mod sql_types {
    use diesel::query_builder::QueryId;
    use diesel::sql_types::SqlType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "geometry"))]
    pub struct Geometry;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "geography"))]
    pub struct Geography;
}

// ---------------------------------------------------------------------------
// Diesel FromSql / ToSql for raw Ewkb<B> (unchanged from original)
// ---------------------------------------------------------------------------

impl<B: AsRef<[u8]> + std::fmt::Debug> ToSql<sql_types::Geometry, Pg> for Ewkb<B> {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        out.write_all(self.0.as_ref())?;
        Ok(IsNull::No)
    }
}

impl<B: AsRef<[u8]> + std::fmt::Debug> ToSql<sql_types::Geography, Pg> for Ewkb<B> {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        out.write_all(self.0.as_ref())?;
        Ok(IsNull::No)
    }
}

impl FromSql<sql_types::Geometry, Pg> for Ewkb<Vec<u8>> {
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        Ok(Self(bytes.as_bytes().to_vec()))
    }
}

impl FromSql<sql_types::Geography, Pg> for Ewkb<Vec<u8>> {
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        Ok(Self(bytes.as_bytes().to_vec()))
    }
}

// ---------------------------------------------------------------------------
// Diesel FromSql / ToSql for wkb::Decode<T> / wkb::Encode<T>
// ---------------------------------------------------------------------------

impl<T: FromWkb + Sized + std::fmt::Debug> FromSql<sql_types::Geometry, Pg> for wkb::Decode<T> {
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut rdr = std::io::Cursor::new(bytes.as_bytes());
        let geom = T::from_wkb(&mut rdr, wkb::WkbDialect::Ewkb)?;
        Ok(wkb::Decode {
            geometry: Some(geom),
        })
    }
}

impl<T: FromWkb + Sized + std::fmt::Debug> FromSql<sql_types::Geography, Pg> for wkb::Decode<T> {
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut rdr = std::io::Cursor::new(bytes.as_bytes());
        let geom = T::from_wkb(&mut rdr, wkb::WkbDialect::Ewkb)?;
        Ok(wkb::Decode {
            geometry: Some(geom),
        })
    }
}

impl<T: GeozeroGeometry + Sized + std::fmt::Debug> ToSql<sql_types::Geometry, Pg>
    for wkb::Encode<T>
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        let mut wkb_out: Vec<u8> = Vec::new();
        let mut writer = wkb::WkbWriter::with_opts(
            &mut wkb_out,
            wkb::WkbDialect::Ewkb,
            self.0.dims(),
            self.0.srid(),
            Vec::new(),
        );
        self.0.process_geom(&mut writer)?;
        out.write_all(&wkb_out)?;
        Ok(IsNull::No)
    }
}

impl<T: GeozeroGeometry + Sized + std::fmt::Debug> ToSql<sql_types::Geography, Pg>
    for wkb::Encode<T>
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        let mut wkb_out: Vec<u8> = Vec::new();
        let mut writer = wkb::WkbWriter::with_opts(
            &mut wkb_out,
            wkb::WkbDialect::Ewkb,
            self.0.dims(),
            self.0.srid(),
            Vec::new(),
        );
        self.0.process_geom(&mut writer)?;
        out.write_all(&wkb_out)?;
        Ok(IsNull::No)
    }
}

// ---------------------------------------------------------------------------
// Diesel macros for user-defined types
// ---------------------------------------------------------------------------

/// Implement Diesel `FromSql<Geometry>` for a type implementing `FromWkb`.
///
/// CAUTION: Does not support decoding NULL values!
#[macro_export]
macro_rules! impl_diesel_postgis_decode {
    ( $t:ty ) => {
        impl
            diesel::deserialize::FromSql<
                $crate::postgis::diesel::sql_types::Geometry,
                diesel::pg::Pg,
            > for $t
        {
            fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
                use $crate::wkb::FromWkb;
                let mut rdr = std::io::Cursor::new(bytes.as_bytes());
                let geom = <$t>::from_wkb(&mut rdr, $crate::wkb::WkbDialect::Ewkb)?;
                Ok(geom)
            }
        }
    };
}

/// Implement Diesel `ToSql<Geometry>` for a type implementing `GeozeroGeometry`.
#[macro_export]
macro_rules! impl_diesel_postgis_encode {
    ( $t:ty ) => {
        impl diesel::serialize::ToSql<$crate::postgis::diesel::sql_types::Geometry, diesel::pg::Pg>
            for $t
        {
            fn to_sql(
                &self,
                out: &mut diesel::serialize::Output<diesel::pg::Pg>,
            ) -> diesel::serialize::Result {
                use std::io::Write;
                use $crate::GeozeroGeometry;
                let mut wkb_out: Vec<u8> = Vec::new();
                let mut writer = $crate::wkb::WkbWriter::with_opts(
                    &mut wkb_out,
                    $crate::wkb::WkbDialect::Ewkb,
                    self.dims(),
                    self.srid(),
                    Vec::new(),
                );
                self.process_geom(&mut writer)?;
                out.write_all(&wkb_out)?;
                Ok(diesel::serialize::IsNull::No)
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn srid_to_i32(srid: Option<u32>) -> Option<i32> {
    srid.map(|s| s as i32)
}

fn dims_from_dimension_flag<P: PointT>(point: &P) -> CoordDimensions {
    let dim = point.dimension();
    CoordDimensions {
        z: dim & 0x8000_0000 != 0,
        m: dim & 0x4000_0000 != 0,
        t: false,
        tm: false,
    }
}

fn process_point<P: PointT, Proc: GeomProcessor>(
    point: &P,
    processor: &mut Proc,
    idx: usize,
) -> Result<()> {
    if processor.multi_dim() {
        processor.coordinate(
            point.get_x(),
            point.get_y(),
            point.get_z(),
            point.get_m(),
            None,
            None,
            idx,
        )
    } else {
        processor.xy(point.get_x(), point.get_y(), idx)
    }
}

fn process_linestring_geom<P: PointT, Proc: GeomProcessor>(
    ls: &LineString<P>,
    tagged: bool,
    idx: usize,
    processor: &mut Proc,
) -> Result<()> {
    processor.linestring_begin(tagged, ls.points.len(), idx)?;
    for (i, point) in ls.points.iter().enumerate() {
        process_point(point, processor, i)?;
    }
    processor.linestring_end(tagged, idx)
}

fn process_polygon_geom<P: PointT, Proc: GeomProcessor>(
    poly: &Polygon<P>,
    tagged: bool,
    idx: usize,
    processor: &mut Proc,
) -> Result<()> {
    processor.polygon_begin(tagged, poly.rings.len(), idx)?;
    for (ring_idx, ring) in poly.rings.iter().enumerate() {
        processor.linestring_begin(false, ring.len(), ring_idx)?;
        for (coord_idx, point) in ring.iter().enumerate() {
            process_point(point, processor, coord_idx)?;
        }
        processor.linestring_end(false, ring_idx)?;
    }
    processor.polygon_end(tagged, idx)
}

fn container_dims<P: PointT>(geom: &GeometryContainer<P>) -> CoordDimensions {
    let dim = geom.dimension();
    CoordDimensions {
        z: dim & 0x8000_0000 != 0,
        m: dim & 0x4000_0000 != 0,
        t: false,
        tm: false,
    }
}

fn container_srid<P: PointT>(geom: &GeometryContainer<P>) -> Option<i32> {
    match geom {
        GeometryContainer::Point(p) => srid_to_i32(p.get_srid()),
        GeometryContainer::LineString(g) => srid_to_i32(g.srid),
        GeometryContainer::Polygon(g) => srid_to_i32(g.srid),
        GeometryContainer::MultiPoint(g) => srid_to_i32(g.srid),
        GeometryContainer::MultiLineString(g) => srid_to_i32(g.srid),
        GeometryContainer::MultiPolygon(g) => srid_to_i32(g.srid),
        GeometryContainer::GeometryCollection(g) => srid_to_i32(g.srid),
    }
}

// ---------------------------------------------------------------------------
// GeozeroGeometry for point types
// ---------------------------------------------------------------------------

macro_rules! impl_geozero_for_point {
    ($($t:ty),+) => {
        $(
            impl GeozeroGeometry for $t {
                fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
                    processor.srid(srid_to_i32(self.get_srid()))?;
                    processor.point_begin(0)?;
                    process_point(self, processor, 0)?;
                    processor.point_end(0)
                }
                fn dims(&self) -> CoordDimensions {
                    dims_from_dimension_flag(self)
                }
                fn srid(&self) -> Option<i32> {
                    srid_to_i32(self.get_srid())
                }
            }
        )+
    };
}

impl_geozero_for_point!(Point, PointZ, PointM, PointZM, AnyPoint);

// ---------------------------------------------------------------------------
// GeozeroGeometry for LineString<P>
// ---------------------------------------------------------------------------

impl<P: PointT> GeozeroGeometry for LineString<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(srid_to_i32(self.srid))?;
        process_linestring_geom(self, true, 0, processor)
    }
    fn dims(&self) -> CoordDimensions {
        self.points
            .first()
            .map(dims_from_dimension_flag)
            .unwrap_or(CoordDimensions::xy())
    }
    fn srid(&self) -> Option<i32> {
        srid_to_i32(self.srid)
    }
}

// ---------------------------------------------------------------------------
// GeozeroGeometry for Polygon<P>
// ---------------------------------------------------------------------------

impl<P: PointT> GeozeroGeometry for Polygon<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(srid_to_i32(self.srid))?;
        process_polygon_geom(self, true, 0, processor)
    }
    fn dims(&self) -> CoordDimensions {
        self.rings
            .first()
            .and_then(|r| r.first())
            .map(dims_from_dimension_flag)
            .unwrap_or(CoordDimensions::xy())
    }
    fn srid(&self) -> Option<i32> {
        srid_to_i32(self.srid)
    }
}

// ---------------------------------------------------------------------------
// GeozeroGeometry for MultiPoint<P>
// ---------------------------------------------------------------------------

impl<P: PointT> GeozeroGeometry for MultiPoint<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(srid_to_i32(self.srid))?;
        processor.multipoint_begin(self.points.len(), 0)?;
        for (i, point) in self.points.iter().enumerate() {
            process_point(point, processor, i)?;
        }
        processor.multipoint_end(0)
    }
    fn dims(&self) -> CoordDimensions {
        self.points
            .first()
            .map(dims_from_dimension_flag)
            .unwrap_or(CoordDimensions::xy())
    }
    fn srid(&self) -> Option<i32> {
        srid_to_i32(self.srid)
    }
}

// ---------------------------------------------------------------------------
// GeozeroGeometry for MultiLineString<P>
// ---------------------------------------------------------------------------

impl<P: PointT> GeozeroGeometry for MultiLineString<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(srid_to_i32(self.srid))?;
        processor.multilinestring_begin(self.lines.len(), 0)?;
        for (i, line) in self.lines.iter().enumerate() {
            process_linestring_geom(line, false, i, processor)?;
        }
        processor.multilinestring_end(0)
    }
    fn dims(&self) -> CoordDimensions {
        self.lines
            .first()
            .and_then(|l| l.points.first())
            .map(dims_from_dimension_flag)
            .unwrap_or(CoordDimensions::xy())
    }
    fn srid(&self) -> Option<i32> {
        srid_to_i32(self.srid)
    }
}

// ---------------------------------------------------------------------------
// GeozeroGeometry for MultiPolygon<P>
// ---------------------------------------------------------------------------

impl<P: PointT> GeozeroGeometry for MultiPolygon<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(srid_to_i32(self.srid))?;
        processor.multipolygon_begin(self.polygons.len(), 0)?;
        for (i, poly) in self.polygons.iter().enumerate() {
            process_polygon_geom(poly, false, i, processor)?;
        }
        processor.multipolygon_end(0)
    }
    fn dims(&self) -> CoordDimensions {
        self.polygons
            .first()
            .and_then(|p| p.rings.first())
            .and_then(|r| r.first())
            .map(dims_from_dimension_flag)
            .unwrap_or(CoordDimensions::xy())
    }
    fn srid(&self) -> Option<i32> {
        srid_to_i32(self.srid)
    }
}

// ---------------------------------------------------------------------------
// GeozeroGeometry for GeometryCollection<P>
// ---------------------------------------------------------------------------

impl<P: PointT> GeozeroGeometry for GeometryCollection<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(srid_to_i32(self.srid))?;
        processor.geometrycollection_begin(self.geometries.len(), 0)?;
        for (i, geom) in self.geometries.iter().enumerate() {
            process_container_n(geom, i, processor)?;
        }
        processor.geometrycollection_end(0)
    }
    fn dims(&self) -> CoordDimensions {
        self.geometries
            .first()
            .map(container_dims)
            .unwrap_or(CoordDimensions::xy())
    }
    fn srid(&self) -> Option<i32> {
        srid_to_i32(self.srid)
    }
}

// ---------------------------------------------------------------------------
// GeozeroGeometry for GeometryContainer<P>
// ---------------------------------------------------------------------------

fn process_container_n<P: PointT, Proc: GeomProcessor>(
    geom: &GeometryContainer<P>,
    idx: usize,
    processor: &mut Proc,
) -> Result<()> {
    match geom {
        GeometryContainer::Point(p) => {
            processor.point_begin(idx)?;
            process_point(p, processor, 0)?;
            processor.point_end(idx)
        }
        GeometryContainer::LineString(ls) => process_linestring_geom(ls, true, idx, processor),
        GeometryContainer::Polygon(poly) => process_polygon_geom(poly, true, idx, processor),
        GeometryContainer::MultiPoint(mp) => {
            processor.multipoint_begin(mp.points.len(), idx)?;
            for (i, point) in mp.points.iter().enumerate() {
                process_point(point, processor, i)?;
            }
            processor.multipoint_end(idx)
        }
        GeometryContainer::MultiLineString(mls) => {
            processor.multilinestring_begin(mls.lines.len(), idx)?;
            for (i, line) in mls.lines.iter().enumerate() {
                process_linestring_geom(line, false, i, processor)?;
            }
            processor.multilinestring_end(idx)
        }
        GeometryContainer::MultiPolygon(mpoly) => {
            processor.multipolygon_begin(mpoly.polygons.len(), idx)?;
            for (i, poly) in mpoly.polygons.iter().enumerate() {
                process_polygon_geom(poly, false, i, processor)?;
            }
            processor.multipolygon_end(idx)
        }
        GeometryContainer::GeometryCollection(gc) => {
            processor.geometrycollection_begin(gc.geometries.len(), idx)?;
            for (i, g) in gc.geometries.iter().enumerate() {
                process_container_n(g, i, processor)?;
            }
            processor.geometrycollection_end(idx)
        }
    }
}

impl<P: PointT> GeozeroGeometry for GeometryContainer<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(container_srid(self))?;
        process_container_n(self, 0, processor)
    }
    fn dims(&self) -> CoordDimensions {
        container_dims(self)
    }
    fn srid(&self) -> Option<i32> {
        container_srid(self)
    }
}

// ---------------------------------------------------------------------------
// PostgisDieselWriter: GeomProcessor that builds GeometryContainer<P>
// ---------------------------------------------------------------------------

/// Generator for postgis_diesel geometry types.
pub struct PostgisDieselWriter<P: PointT> {
    geom: Option<GeometryContainer<P>>,
    srid: Option<u32>,
    /// Stack of in-progress geometry collections
    collections: Vec<Vec<GeometryContainer<P>>>,
    /// In-progress multi-polygon
    polygons: Option<Vec<Polygon<P>>>,
    /// In-progress polygon rings or multi-linestring lines
    line_strings: Option<Vec<Vec<P>>>,
    /// In-progress point or line_string coordinates
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
        let coords = self
            .coords
            .as_mut()
            .ok_or(GeozeroError::Geometry("Not ready for coords".to_string()))?;
        coords.push(point);
        Ok(())
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
        let coords = self
            .coords
            .as_mut()
            .ok_or(GeozeroError::Geometry("Not ready for coords".to_string()))?;
        coords.push(point);
        Ok(())
    }

    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        self.coords = Some(Vec::with_capacity(1));
        Ok(())
    }

    fn point_end(&mut self, _idx: usize) -> Result<()> {
        let coords = self
            .coords
            .take()
            .ok_or(GeozeroError::Geometry("No coords for Point".to_string()))?;
        let point = coords
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
        let coords = self.coords.take().ok_or(GeozeroError::Geometry(
            "No coords for MultiPoint".to_string(),
        ))?;
        self.finish_geometry(GeometryContainer::MultiPoint(MultiPoint {
            points: coords,
            srid: self.srid,
        }))
    }

    fn linestring_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        self.coords = Some(Vec::with_capacity(size));
        Ok(())
    }

    fn linestring_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        let coords = self.coords.take().ok_or(GeozeroError::Geometry(
            "No coords for LineString".to_string(),
        ))?;
        if tagged {
            self.finish_geometry(GeometryContainer::LineString(LineString {
                points: coords,
                srid: self.srid,
            }))?;
        } else {
            let line_strings = self.line_strings.as_mut().ok_or(GeozeroError::Geometry(
                "Missing container for LineString".to_string(),
            ))?;
            line_strings.push(coords);
        }
        Ok(())
    }

    fn multilinestring_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.line_strings = Some(Vec::with_capacity(size));
        Ok(())
    }

    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        let rings = self.line_strings.take().ok_or(GeozeroError::Geometry(
            "No lines for MultiLineString".to_string(),
        ))?;
        let lines = rings
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
            self.finish_geometry(GeometryContainer::Polygon(polygon))?;
        } else {
            let polygons = self.polygons.as_mut().ok_or(GeozeroError::Geometry(
                "Missing container for Polygon".to_string(),
            ))?;
            polygons.push(polygon);
        }
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

// ---------------------------------------------------------------------------
// FromWkb for GeometryContainer<P>
// ---------------------------------------------------------------------------

macro_rules! impl_from_wkb {
    ($p:ty) => {
        impl FromWkb for GeometryContainer<$p> {
            fn from_wkb<R: Read>(rdr: &mut R, dialect: wkb::WkbDialect) -> Result<Self> {
                let mut writer = PostgisDieselWriter::<$p>::new();
                wkb::process_wkb_type_geom(rdr, &mut writer, dialect)?;
                writer
                    .take_geometry()
                    .ok_or(GeozeroError::Geometry("Missing geometry".to_string()))
            }
        }
    };
}

impl_from_wkb!(Point);
impl_from_wkb!(PointZ);
impl_from_wkb!(PointM);
impl_from_wkb!(PointZM);
impl_from_wkb!(AnyPoint);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToWkt;
    use crate::wkb::conversion::ToWkb;

    #[test]
    fn point_to_wkt() {
        let point = Point::new(1.0, 2.0, None);
        assert_eq!(point.to_wkt().unwrap(), "POINT(1 2)");
    }

    #[test]
    fn point_z_to_wkt() {
        let point = PointZ::new(1.0, 2.0, 3.0, Some(4326));
        assert_eq!(
            point.to_wkt_ndim(CoordDimensions::xyz()).unwrap(),
            "POINT(1 2 3)"
        );
    }

    #[test]
    fn point_m_to_wkt() {
        let point = PointM::new(1.0, 2.0, 3.0, None);
        let dims = CoordDimensions {
            z: false,
            m: true,
            t: false,
            tm: false,
        };
        // M value is emitted as a third coordinate when M dimension is enabled
        assert_eq!(point.to_wkt_ndim(dims).unwrap(), "POINT(1 2 3)");
    }

    #[test]
    fn point_zm_to_wkt() {
        let point = PointZM::new(1.0, 2.0, 3.0, 4.0, None);
        assert_eq!(
            point.to_wkt_ndim(CoordDimensions::xyzm()).unwrap(),
            "POINT(1 2 3 4)"
        );
    }

    #[test]
    fn anypoint_to_wkt() {
        let point = AnyPoint::PointZ(PointZ::new(1.0, 2.0, 3.0, Some(7415)));
        assert_eq!(
            point.to_wkt_ndim(CoordDimensions::xyz()).unwrap(),
            "POINT(1 2 3)"
        );
    }

    #[test]
    fn linestring_to_wkt() {
        let mut ls = LineString::<Point>::new(None);
        ls.add_point(Point::new(0.0, 0.0, None)).unwrap();
        ls.add_point(Point::new(1.0, 1.0, None)).unwrap();
        assert_eq!(ls.to_wkt().unwrap(), "LINESTRING(0 0,1 1)");
    }

    #[test]
    fn linestring_z_to_wkt() {
        let mut ls = LineString::<PointZ>::new(Some(4326));
        ls.add_point(PointZ::new(0.0, 0.0, 10.0, Some(4326)))
            .unwrap();
        ls.add_point(PointZ::new(1.0, 1.0, 20.0, Some(4326)))
            .unwrap();
        assert_eq!(
            ls.to_wkt_ndim(CoordDimensions::xyz()).unwrap(),
            "LINESTRING(0 0 10,1 1 20)"
        );
    }

    #[test]
    fn polygon_to_wkt() {
        let mut poly = Polygon::<Point>::new(None);
        poly.add_point(Point::new(0.0, 0.0, None)).unwrap();
        poly.add_point(Point::new(4.0, 0.0, None)).unwrap();
        poly.add_point(Point::new(4.0, 4.0, None)).unwrap();
        poly.add_point(Point::new(0.0, 4.0, None)).unwrap();
        poly.add_point(Point::new(0.0, 0.0, None)).unwrap();
        assert_eq!(poly.to_wkt().unwrap(), "POLYGON((0 0,4 0,4 4,0 4,0 0))");
    }

    #[test]
    fn polygon_with_hole_to_wkt() {
        let mut poly = Polygon::<Point>::new(None);
        // exterior ring
        poly.add_point(Point::new(0.0, 0.0, None)).unwrap();
        poly.add_point(Point::new(10.0, 0.0, None)).unwrap();
        poly.add_point(Point::new(10.0, 10.0, None)).unwrap();
        poly.add_point(Point::new(0.0, 10.0, None)).unwrap();
        poly.add_point(Point::new(0.0, 0.0, None)).unwrap();
        // interior ring
        poly.add_ring();
        poly.add_point(Point::new(1.0, 1.0, None)).unwrap();
        poly.add_point(Point::new(2.0, 1.0, None)).unwrap();
        poly.add_point(Point::new(2.0, 2.0, None)).unwrap();
        poly.add_point(Point::new(1.0, 2.0, None)).unwrap();
        poly.add_point(Point::new(1.0, 1.0, None)).unwrap();
        assert_eq!(
            poly.to_wkt().unwrap(),
            "POLYGON((0 0,10 0,10 10,0 10,0 0),(1 1,2 1,2 2,1 2,1 1))"
        );
    }

    #[test]
    fn multipoint_to_wkt() {
        let mut mp = MultiPoint::<Point>::new(None);
        mp.add_point(Point::new(1.0, 2.0, None));
        mp.add_point(Point::new(3.0, 4.0, None));
        assert_eq!(mp.to_wkt().unwrap(), "MULTIPOINT(1 2,3 4)");
    }

    #[test]
    fn multilinestring_to_wkt() {
        let mut mls = MultiLineString::<Point>::new(None);
        mls.add_line();
        mls.add_point(Point::new(0.0, 0.0, None)).unwrap();
        mls.add_point(Point::new(1.0, 1.0, None)).unwrap();
        mls.add_line();
        mls.add_point(Point::new(2.0, 2.0, None)).unwrap();
        mls.add_point(Point::new(3.0, 3.0, None)).unwrap();
        assert_eq!(
            mls.to_wkt().unwrap(),
            "MULTILINESTRING((0 0,1 1),(2 2,3 3))"
        );
    }

    #[test]
    fn multipolygon_to_wkt() {
        let mut mpoly = MultiPolygon::<Point>::new(None);
        mpoly.add_empty_polygon();
        mpoly.add_point(Point::new(0.0, 0.0, None)).unwrap();
        mpoly.add_point(Point::new(1.0, 0.0, None)).unwrap();
        mpoly.add_point(Point::new(1.0, 1.0, None)).unwrap();
        mpoly.add_point(Point::new(0.0, 0.0, None)).unwrap();
        assert_eq!(mpoly.to_wkt().unwrap(), "MULTIPOLYGON(((0 0,1 0,1 1,0 0)))");
    }

    #[test]
    fn geometry_collection_to_wkt() {
        let mut gc = GeometryCollection::<Point>::new(None);
        gc.add_geometry(GeometryContainer::Point(Point::new(1.0, 2.0, None)));
        let mut ls = LineString::<Point>::new(None);
        ls.add_point(Point::new(0.0, 0.0, None)).unwrap();
        ls.add_point(Point::new(1.0, 1.0, None)).unwrap();
        gc.add_geometry(GeometryContainer::LineString(ls));
        assert_eq!(
            gc.to_wkt().unwrap(),
            "GEOMETRYCOLLECTION(POINT(1 2),LINESTRING(0 0,1 1))"
        );
    }

    #[test]
    fn geometry_container_point_to_wkt() {
        let gc = GeometryContainer::Point(Point::new(1.0, 2.0, None));
        assert_eq!(gc.to_wkt().unwrap(), "POINT(1 2)");
    }

    #[test]
    fn geometry_container_polygon_to_wkt() {
        let mut poly = Polygon::<Point>::new(None);
        poly.add_point(Point::new(0.0, 0.0, None)).unwrap();
        poly.add_point(Point::new(1.0, 0.0, None)).unwrap();
        poly.add_point(Point::new(1.0, 1.0, None)).unwrap();
        poly.add_point(Point::new(0.0, 0.0, None)).unwrap();
        let gc = GeometryContainer::Polygon(poly);
        assert_eq!(gc.to_wkt().unwrap(), "POLYGON((0 0,1 0,1 1,0 0))");
    }

    #[cfg(feature = "with-geojson")]
    #[test]
    fn point_to_geojson() {
        use crate::geojson::conversion::ToJson;
        let point = Point::new(1.0, 2.0, Some(4326));
        let json = point.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "Point");
        assert_eq!(parsed["coordinates"][0], 1.0);
        assert_eq!(parsed["coordinates"][1], 2.0);
    }

    #[cfg(feature = "with-geojson")]
    #[test]
    fn point_z_to_geojson() {
        // GeoJSON writer with Z support
        use crate::geojson::GeoJsonWriter;
        let point = PointZ::new(1.0, 2.0, 3.0, Some(7415));
        let mut out: Vec<u8> = Vec::new();
        let mut writer = GeoJsonWriter::with_dims(&mut out, CoordDimensions::xyz());
        point.process_geom(&mut writer).unwrap();
        let json = String::from_utf8(out).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "Point");
        assert_eq!(parsed["coordinates"][2], 3.0);
    }

    #[cfg(feature = "with-geojson")]
    #[test]
    fn polygon_to_geojson() {
        use crate::geojson::conversion::ToJson;
        let mut poly = Polygon::<Point>::new(Some(4326));
        poly.add_point(Point::new(0.0, 0.0, Some(4326))).unwrap();
        poly.add_point(Point::new(1.0, 0.0, Some(4326))).unwrap();
        poly.add_point(Point::new(1.0, 1.0, Some(4326))).unwrap();
        poly.add_point(Point::new(0.0, 0.0, Some(4326))).unwrap();
        let json = poly.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "Polygon");
        assert_eq!(parsed["coordinates"][0][0][0], 0.0);
    }

    #[test]
    fn point_to_ewkb_roundtrip() {
        let point = Point::new(1.0, 2.0, Some(4326));
        let ewkb = point.to_ewkb(CoordDimensions::xy(), Some(4326)).unwrap();
        let container =
            GeometryContainer::<Point>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::Point(p) => {
                assert_eq!(p.x, 1.0);
                assert_eq!(p.y, 2.0);
                assert_eq!(p.srid, Some(4326));
            }
            _ => panic!("Expected Point"),
        }
    }

    #[test]
    fn point_z_to_ewkb_roundtrip() {
        let point = PointZ::new(1.0, 2.0, 3.0, Some(4326));
        let ewkb = point.to_ewkb(point.dims(), point.srid()).unwrap();
        let container =
            GeometryContainer::<PointZ>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::Point(p) => {
                assert_eq!(p.x, 1.0);
                assert_eq!(p.y, 2.0);
                assert_eq!(p.z, 3.0);
                assert_eq!(p.srid, Some(4326));
            }
            _ => panic!("Expected PointZ"),
        }
    }

    #[test]
    fn anypoint_ewkb_roundtrip() {
        let point = AnyPoint::PointZ(PointZ::new(1.0, 2.0, 3.0, Some(4326)));
        let ewkb = point.to_ewkb(point.dims(), point.srid()).unwrap();
        let container =
            GeometryContainer::<AnyPoint>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::Point(AnyPoint::PointZ(p)) => {
                assert_eq!(p.x, 1.0);
                assert_eq!(p.y, 2.0);
                assert_eq!(p.z, 3.0);
                assert_eq!(p.srid, Some(4326));
            }
            _ => panic!("Expected AnyPoint::PointZ"),
        }
    }

    #[test]
    fn polygon_ewkb_roundtrip() {
        let mut poly = Polygon::<Point>::new(Some(4326));
        poly.add_point(Point::new(0.0, 0.0, Some(4326))).unwrap();
        poly.add_point(Point::new(1.0, 0.0, Some(4326))).unwrap();
        poly.add_point(Point::new(1.0, 1.0, Some(4326))).unwrap();
        poly.add_point(Point::new(0.0, 0.0, Some(4326))).unwrap();

        let ewkb = poly.to_ewkb(CoordDimensions::xy(), Some(4326)).unwrap();
        let container =
            GeometryContainer::<Point>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::Polygon(p) => {
                assert_eq!(p.rings.len(), 1);
                assert_eq!(p.rings[0].len(), 4);
                assert_eq!(p.srid, Some(4326));
            }
            _ => panic!("Expected Polygon"),
        }
    }

    #[cfg(feature = "with-geojson")]
    #[test]
    fn geojson_to_postgis_diesel_2d() {
        use crate::geojson::GeoJsonString;
        let geojson = GeoJsonString(r#"{"type":"Point","coordinates":[1.0,2.0]}"#.to_string());
        let ewkb = geojson.to_ewkb(CoordDimensions::xy(), None).unwrap();
        let container =
            GeometryContainer::<Point>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::Point(p) => {
                assert_eq!(p.x, 1.0);
                assert_eq!(p.y, 2.0);
            }
            other => panic!("Expected Point, got {other:?}"),
        }
    }

    #[cfg(feature = "with-geojson")]
    #[test]
    fn geojson_to_postgis_diesel_3d() {
        use crate::geojson::GeoJsonString;
        let geojson = GeoJsonString(r#"{"type":"Point","coordinates":[1.0,2.0,3.0]}"#.to_string());
        let ewkb = geojson.to_ewkb(CoordDimensions::xyz(), None).unwrap();
        let container =
            GeometryContainer::<AnyPoint>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::Point(AnyPoint::PointZ(p)) => {
                assert_eq!(p.x, 1.0);
                assert_eq!(p.y, 2.0);
                assert_eq!(p.z, 3.0);
            }
            other => panic!("Expected AnyPoint::PointZ, got {other:?}"),
        }
    }

    #[cfg(feature = "with-geojson")]
    #[test]
    fn geojson_polygon_to_postgis_diesel() {
        use crate::geojson::GeoJsonString;
        let geojson = GeoJsonString(
            r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,0]]]}"#.to_string(),
        );
        let ewkb = geojson.to_ewkb(CoordDimensions::xy(), None).unwrap();
        let container =
            GeometryContainer::<Point>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::Polygon(poly) => {
                assert_eq!(poly.rings.len(), 1);
                assert_eq!(poly.rings[0].len(), 4);
                assert_eq!(poly.rings[0][0].x, 0.0);
                assert_eq!(poly.rings[0][1].x, 1.0);
            }
            other => panic!("Expected Polygon, got {other:?}"),
        }
    }

    #[test]
    fn srid_preserved_in_geozero_geometry() {
        let point = Point::new(1.0, 2.0, Some(4326));
        assert_eq!(GeozeroGeometry::srid(&point), Some(4326));
        let dims = GeozeroGeometry::dims(&point);
        assert!(!dims.z && !dims.m);

        let point_z = PointZ::new(1.0, 2.0, 3.0, Some(7415));
        assert_eq!(GeozeroGeometry::srid(&point_z), Some(7415));
        let dims = GeozeroGeometry::dims(&point_z);
        assert!(dims.z && !dims.m);
    }

    #[test]
    fn multipoint_ewkb_roundtrip() {
        let mut mp = MultiPoint::<Point>::new(Some(4326));
        mp.add_point(Point::new(1.0, 2.0, Some(4326)));
        mp.add_point(Point::new(3.0, 4.0, Some(4326)));

        let ewkb = mp.to_ewkb(mp.dims(), mp.srid()).unwrap();
        let container =
            GeometryContainer::<Point>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::MultiPoint(result) => {
                assert_eq!(result.points.len(), 2);
                assert_eq!(result.points[0].x, 1.0);
                assert_eq!(result.points[1].x, 3.0);
                assert_eq!(result.srid, Some(4326));
            }
            other => panic!("Expected MultiPoint, got {other:?}"),
        }
    }

    #[test]
    fn multilinestring_ewkb_roundtrip() {
        let mut mls = MultiLineString::<Point>::new(Some(4326));
        mls.add_line();
        mls.add_point(Point::new(0.0, 0.0, Some(4326))).unwrap();
        mls.add_point(Point::new(1.0, 1.0, Some(4326))).unwrap();
        mls.add_line();
        mls.add_point(Point::new(2.0, 2.0, Some(4326))).unwrap();
        mls.add_point(Point::new(3.0, 3.0, Some(4326))).unwrap();

        let ewkb = mls.to_ewkb(mls.dims(), mls.srid()).unwrap();
        let container =
            GeometryContainer::<Point>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::MultiLineString(result) => {
                assert_eq!(result.lines.len(), 2);
                assert_eq!(result.lines[0].points.len(), 2);
                assert_eq!(result.lines[1].points[0].x, 2.0);
                assert_eq!(result.srid, Some(4326));
            }
            other => panic!("Expected MultiLineString, got {other:?}"),
        }
    }

    #[test]
    fn multipolygon_ewkb_roundtrip() {
        let mut mpoly = MultiPolygon::<Point>::new(Some(4326));
        mpoly.add_empty_polygon();
        mpoly.add_point(Point::new(0.0, 0.0, Some(4326))).unwrap();
        mpoly.add_point(Point::new(1.0, 0.0, Some(4326))).unwrap();
        mpoly.add_point(Point::new(1.0, 1.0, Some(4326))).unwrap();
        mpoly.add_point(Point::new(0.0, 0.0, Some(4326))).unwrap();

        let ewkb = mpoly.to_ewkb(mpoly.dims(), mpoly.srid()).unwrap();
        let container =
            GeometryContainer::<Point>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::MultiPolygon(result) => {
                assert_eq!(result.polygons.len(), 1);
                assert_eq!(result.polygons[0].rings.len(), 1);
                assert_eq!(result.polygons[0].rings[0].len(), 4);
                assert_eq!(result.srid, Some(4326));
            }
            other => panic!("Expected MultiPolygon, got {other:?}"),
        }
    }

    #[test]
    fn geometry_collection_ewkb_roundtrip() {
        let mut gc = GeometryCollection::<Point>::new(Some(4326));
        gc.add_geometry(GeometryContainer::Point(Point::new(1.0, 2.0, Some(4326))));
        let mut ls = LineString::<Point>::new(Some(4326));
        ls.add_point(Point::new(0.0, 0.0, Some(4326))).unwrap();
        ls.add_point(Point::new(1.0, 1.0, Some(4326))).unwrap();
        gc.add_geometry(GeometryContainer::LineString(ls));

        let gc_container = GeometryContainer::GeometryCollection(gc);
        let ewkb = gc_container
            .to_ewkb(gc_container.dims(), gc_container.srid())
            .unwrap();
        let result =
            GeometryContainer::<Point>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match result {
            GeometryContainer::GeometryCollection(gc) => {
                assert_eq!(gc.geometries.len(), 2);
                assert_eq!(gc.srid, Some(4326));
                match &gc.geometries[0] {
                    GeometryContainer::Point(p) => {
                        assert_eq!(p.x, 1.0);
                        assert_eq!(p.y, 2.0);
                    }
                    other => panic!("Expected Point, got {other:?}"),
                }
                match &gc.geometries[1] {
                    GeometryContainer::LineString(ls) => {
                        assert_eq!(ls.points.len(), 2);
                    }
                    other => panic!("Expected LineString, got {other:?}"),
                }
            }
            other => panic!("Expected GeometryCollection, got {other:?}"),
        }
    }

    #[test]
    fn linestring_ewkb_roundtrip() {
        let mut ls = LineString::<Point>::new(Some(4326));
        ls.add_point(Point::new(0.0, 0.0, Some(4326))).unwrap();
        ls.add_point(Point::new(1.0, 1.0, Some(4326))).unwrap();
        ls.add_point(Point::new(2.0, 3.0, Some(4326))).unwrap();

        let ewkb = ls.to_ewkb(ls.dims(), ls.srid()).unwrap();
        let container =
            GeometryContainer::<Point>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::LineString(result) => {
                assert_eq!(result.points.len(), 3);
                assert_eq!(result.points[2].x, 2.0);
                assert_eq!(result.points[2].y, 3.0);
                assert_eq!(result.srid, Some(4326));
            }
            other => panic!("Expected LineString, got {other:?}"),
        }
    }

    #[cfg(feature = "with-geojson")]
    #[test]
    fn geojson_multipolygon_to_postgis_diesel() {
        use crate::geojson::GeoJsonString;
        let geojson = GeoJsonString(
            r#"{"type":"MultiPolygon","coordinates":[[[[0,0],[1,0],[1,1],[0,0]]],[[[2,2],[3,2],[3,3],[2,2]]]]}"#
                .to_string(),
        );
        let ewkb = geojson.to_ewkb(CoordDimensions::xy(), None).unwrap();
        let container =
            GeometryContainer::<Point>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::MultiPolygon(mp) => {
                assert_eq!(mp.polygons.len(), 2);
                assert_eq!(mp.polygons[0].rings[0][0].x, 0.0);
                assert_eq!(mp.polygons[1].rings[0][0].x, 2.0);
            }
            other => panic!("Expected MultiPolygon, got {other:?}"),
        }
    }

    #[test]
    fn point_m_ewkb_roundtrip() {
        let point = PointM::new(1.0, 2.0, 3.0, Some(4326));
        let ewkb = point.to_ewkb(point.dims(), point.srid()).unwrap();
        let container =
            GeometryContainer::<PointM>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::Point(p) => {
                assert_eq!(p.x, 1.0);
                assert_eq!(p.y, 2.0);
                assert_eq!(p.m, 3.0);
                assert_eq!(p.srid, Some(4326));
            }
            other => panic!("Expected PointM, got {other:?}"),
        }
    }

    #[test]
    fn point_zm_ewkb_roundtrip() {
        let point = PointZM::new(1.0, 2.0, 3.0, 4.0, Some(4326));
        let ewkb = point.to_ewkb(point.dims(), point.srid()).unwrap();
        let container =
            GeometryContainer::<PointZM>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::Point(p) => {
                assert_eq!(p.x, 1.0);
                assert_eq!(p.y, 2.0);
                assert_eq!(p.z, 3.0);
                assert_eq!(p.m, 4.0);
                assert_eq!(p.srid, Some(4326));
            }
            other => panic!("Expected PointZM, got {other:?}"),
        }
    }

    #[test]
    fn container_delegates_to_inner_type() {
        let container = GeometryContainer::Point(PointZ::new(1.0, 2.0, 3.0, Some(4326)));
        assert_eq!(
            container.to_wkt_ndim(CoordDimensions::xyz()).unwrap(),
            "POINT(1 2 3)"
        );
        assert_eq!(GeozeroGeometry::srid(&container), Some(4326));
        let dims = GeozeroGeometry::dims(&container);
        assert!(dims.z && !dims.m);
    }
}
