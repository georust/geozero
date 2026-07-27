use postgis_diesel::types::{
    AnyPoint, GeometryCollection, GeometryContainer, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, PointM, PointT, PointZ, PointZM, Polygon,
};

use crate::error::Result;
use crate::{CoordDimensions, GeomProcessor, GeozeroGeometry};

/// postgis_diesel encodes Z and M presence in the high bits of its dimension flag.
const HAS_Z: u32 = 0x8000_0000;
const HAS_M: u32 = 0x4000_0000;

fn dims_from_flag(dim: u32) -> CoordDimensions {
    CoordDimensions {
        z: dim & HAS_Z != 0,
        m: dim & HAS_M != 0,
        t: false,
        tm: false,
    }
}

fn srid_to_i32(srid: Option<u32>) -> Option<i32> {
    srid.map(|s| s as i32)
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

fn process_linestring<P: PointT, Proc: GeomProcessor>(
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

fn process_polygon<P: PointT, Proc: GeomProcessor>(
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

/// Emits `geom` at position `idx` without a leading `srid` call, so it can be used both for a
/// top-level geometry and for a member of a geometry collection.
fn process_container<P: PointT, Proc: GeomProcessor>(
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
        GeometryContainer::LineString(ls) => process_linestring(ls, true, idx, processor),
        GeometryContainer::Polygon(poly) => process_polygon(poly, true, idx, processor),
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
                process_linestring(line, false, i, processor)?;
            }
            processor.multilinestring_end(idx)
        }
        GeometryContainer::MultiPolygon(mpoly) => {
            processor.multipolygon_begin(mpoly.polygons.len(), idx)?;
            for (i, poly) in mpoly.polygons.iter().enumerate() {
                process_polygon(poly, false, i, processor)?;
            }
            processor.multipolygon_end(idx)
        }
        GeometryContainer::GeometryCollection(gc) => {
            processor.geometrycollection_begin(gc.geometries.len(), idx)?;
            for (i, g) in gc.geometries.iter().enumerate() {
                process_container(g, i, processor)?;
            }
            processor.geometrycollection_end(idx)
        }
    }
}

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
                    dims_from_flag(self.dimension())
                }
                fn srid(&self) -> Option<i32> {
                    srid_to_i32(self.get_srid())
                }
            }
        )+
    };
}

impl_geozero_for_point!(Point, PointZ, PointM, PointZM, AnyPoint);

impl<P: PointT> GeozeroGeometry for LineString<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(srid_to_i32(self.srid))?;
        process_linestring(self, true, 0, processor)
    }
    fn dims(&self) -> CoordDimensions {
        dims_from_flag(self.dimension())
    }
    fn srid(&self) -> Option<i32> {
        srid_to_i32(self.srid)
    }
}

impl<P: PointT> GeozeroGeometry for Polygon<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(srid_to_i32(self.srid))?;
        process_polygon(self, true, 0, processor)
    }
    fn dims(&self) -> CoordDimensions {
        dims_from_flag(self.dimension())
    }
    fn srid(&self) -> Option<i32> {
        srid_to_i32(self.srid)
    }
}

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
        dims_from_flag(self.dimension())
    }
    fn srid(&self) -> Option<i32> {
        srid_to_i32(self.srid)
    }
}

impl<P: PointT> GeozeroGeometry for MultiLineString<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(srid_to_i32(self.srid))?;
        processor.multilinestring_begin(self.lines.len(), 0)?;
        for (i, line) in self.lines.iter().enumerate() {
            process_linestring(line, false, i, processor)?;
        }
        processor.multilinestring_end(0)
    }
    fn dims(&self) -> CoordDimensions {
        dims_from_flag(self.dimension())
    }
    fn srid(&self) -> Option<i32> {
        srid_to_i32(self.srid)
    }
}

impl<P: PointT> GeozeroGeometry for MultiPolygon<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(srid_to_i32(self.srid))?;
        processor.multipolygon_begin(self.polygons.len(), 0)?;
        for (i, poly) in self.polygons.iter().enumerate() {
            process_polygon(poly, false, i, processor)?;
        }
        processor.multipolygon_end(0)
    }
    fn dims(&self) -> CoordDimensions {
        dims_from_flag(self.dimension())
    }
    fn srid(&self) -> Option<i32> {
        srid_to_i32(self.srid)
    }
}

impl<P: PointT> GeozeroGeometry for GeometryCollection<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(srid_to_i32(self.srid))?;
        processor.geometrycollection_begin(self.geometries.len(), 0)?;
        for (i, geom) in self.geometries.iter().enumerate() {
            process_container(geom, i, processor)?;
        }
        processor.geometrycollection_end(0)
    }
    fn dims(&self) -> CoordDimensions {
        dims_from_flag(self.dimension())
    }
    fn srid(&self) -> Option<i32> {
        srid_to_i32(self.srid)
    }
}

impl<P: PointT> GeozeroGeometry for GeometryContainer<P> {
    fn process_geom<Proc: GeomProcessor>(&self, processor: &mut Proc) -> Result<()> {
        processor.srid(self.srid())?;
        process_container(self, 0, processor)
    }
    fn dims(&self) -> CoordDimensions {
        dims_from_flag(self.dimension())
    }
    fn srid(&self) -> Option<i32> {
        let srid = match self {
            GeometryContainer::Point(g) => g.get_srid(),
            GeometryContainer::LineString(g) => g.srid,
            GeometryContainer::Polygon(g) => g.srid,
            GeometryContainer::MultiPoint(g) => g.srid,
            GeometryContainer::MultiLineString(g) => g.srid,
            GeometryContainer::MultiPolygon(g) => g.srid,
            GeometryContainer::GeometryCollection(g) => g.srid,
        };
        srid_to_i32(srid)
    }
}
