use crate::error::Result;
use crate::{GeozeroGeometry, ToWkt};
use std::fmt;
use std::io::Read;

/// Encode to WKB
// Used to impl encoding for foreign types
pub struct Encode<T: GeozeroGeometry>(pub T);

/// Decode from WKB
// Used to impl decoding for foreign types
pub struct Decode<T: FromWkb> {
    /// Decoded geometry. `None` for `NULL` value.
    pub geometry: Option<T>,
}

// required by postgres ToSql
impl<T: GeozeroGeometry + Sized> fmt::Debug for Encode<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_wkt().unwrap_or("<unknown geometry>".to_string()))
    }
}

// required by SQLx macros
impl<T: FromWkb + Sized> fmt::Debug for Decode<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<geometry>")
    }
}

/// Convert from WKB.
pub trait FromWkb {
    /// Convert from WKB.
    fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> Result<Self>
    where
        Self: Sized;
}

/// WKB dialect.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum WkbDialect {
    Wkb,
    Ewkb,
    Geopackage,
    MySQL,
    SpatiaLite,
}

/// WKB Types according to OGC 06-103r4 (<https://www.ogc.org/standards/sfa>)
#[derive(PartialEq, Clone, Debug)]
pub enum WKBGeometryType {
    Unknown = 0,
    Point = 1,
    LineString = 2,
    Polygon = 3,
    MultiPoint = 4,
    MultiLineString = 5,
    MultiPolygon = 6,
    GeometryCollection = 7,
    CircularString = 8,
    CompoundCurve = 9,
    CurvePolygon = 10,
    MultiCurve = 11,
    MultiSurface = 12,
    Curve = 13,
    Surface = 14,
    PolyhedralSurface = 15,
    Tin = 16,
    Triangle = 17,
    PointZ = 1001,
    LineStringZ = 1002,
    PolygonZ = 1003,
    MultiPointZ = 1004,
    MultiLineStringZ = 1005,
    MultiPolygonZ = 1006,
    GeometryCollectionZ = 1007,
    CircularStringZ = 1008,
    CompoundCurveZ = 1009,
    CurvePolygonZ = 1010,
    MultiCurveZ = 1011,
    MultiSurfaceZ = 1012,
    CurveZ = 1013,
    SurfaceZ = 1014,
    PolyhedralSurfaceZ = 1015,
    TinZ = 1016,
    TriangleZ = 1017,
    PointM = 2001,
    LineStringM = 2002,
    PolygonM = 2003,
    MultiPointM = 2004,
    MultiLineStringM = 2005,
    MultiPolygonM = 2006,
    GeometryCollectionM = 2007,
    CircularStringM = 2008,
    CompoundCurveM = 2009,
    CurvePolygonM = 2010,
    MultiCurveM = 2011,
    MultiSurfaceM = 2012,
    CurveM = 2013,
    SurfaceM = 2014,
    PolyhedralSurfaceM = 2015,
    TinM = 2016,
    TriangleM = 2017,
    PointZM = 3001,
    LineStringZM = 3002,
    PolygonZM = 3003,
    MultiPointZM = 3004,
    MultiLineStringZM = 3005,
    MultiPolygonZM = 3006,
    GeometryCollectionZM = 3007,
    CircularStringZM = 3008,
    CompoundCurveZM = 3009,
    CurvePolygonZM = 3010,
    MultiCurveZM = 3011,
    MultiSurfaceZM = 3012,
    CurveZM = 3013,
    SurfaceZM = 3014,
    PolyhedralSurfaceZM = 3015,
    TinZM = 3016,
    TriangleZM = 3017,
}

impl WKBGeometryType {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => WKBGeometryType::Point,
            2 => WKBGeometryType::LineString,
            3 => WKBGeometryType::Polygon,
            4 => WKBGeometryType::MultiPoint,
            5 => WKBGeometryType::MultiLineString,
            6 => WKBGeometryType::MultiPolygon,
            7 => WKBGeometryType::GeometryCollection,
            8 => WKBGeometryType::CircularString,
            9 => WKBGeometryType::CompoundCurve,
            10 => WKBGeometryType::CurvePolygon,
            11 => WKBGeometryType::MultiCurve,
            12 => WKBGeometryType::MultiSurface,
            13 => WKBGeometryType::Curve,
            14 => WKBGeometryType::Surface,
            15 => WKBGeometryType::PolyhedralSurface,
            16 => WKBGeometryType::Tin,
            17 => WKBGeometryType::Triangle,
            1001 => WKBGeometryType::PointZ,
            1002 => WKBGeometryType::LineStringZ,
            1003 => WKBGeometryType::PolygonZ,
            1004 => WKBGeometryType::MultiPointZ,
            1005 => WKBGeometryType::MultiLineStringZ,
            1006 => WKBGeometryType::MultiPolygonZ,
            1007 => WKBGeometryType::GeometryCollectionZ,
            1008 => WKBGeometryType::CircularStringZ,
            1009 => WKBGeometryType::CompoundCurveZ,
            1010 => WKBGeometryType::CurvePolygonZ,
            1011 => WKBGeometryType::MultiCurveZ,
            1012 => WKBGeometryType::MultiSurfaceZ,
            1013 => WKBGeometryType::CurveZ,
            1014 => WKBGeometryType::SurfaceZ,
            1015 => WKBGeometryType::PolyhedralSurfaceZ,
            1016 => WKBGeometryType::TinZ,
            1017 => WKBGeometryType::TriangleZ,
            2001 => WKBGeometryType::PointM,
            2002 => WKBGeometryType::LineStringM,
            2003 => WKBGeometryType::PolygonM,
            2017 => WKBGeometryType::TriangleM,
            2004 => WKBGeometryType::MultiPointM,
            2005 => WKBGeometryType::MultiLineStringM,
            2006 => WKBGeometryType::MultiPolygonM,
            2007 => WKBGeometryType::GeometryCollectionM,
            2008 => WKBGeometryType::CircularStringM,
            2009 => WKBGeometryType::CompoundCurveM,
            2010 => WKBGeometryType::CurvePolygonM,
            2011 => WKBGeometryType::MultiCurveM,
            2012 => WKBGeometryType::MultiSurfaceM,
            2013 => WKBGeometryType::CurveM,
            2014 => WKBGeometryType::SurfaceM,
            2015 => WKBGeometryType::PolyhedralSurfaceM,
            2016 => WKBGeometryType::TinM,
            3001 => WKBGeometryType::PointZM,
            3002 => WKBGeometryType::LineStringZM,
            3003 => WKBGeometryType::PolygonZM,
            3004 => WKBGeometryType::MultiPointZM,
            3005 => WKBGeometryType::MultiLineStringZM,
            3006 => WKBGeometryType::MultiPolygonZM,
            3007 => WKBGeometryType::GeometryCollectionZM,
            3008 => WKBGeometryType::CircularStringZM,
            3009 => WKBGeometryType::CompoundCurveZM,
            3010 => WKBGeometryType::CurvePolygonZM,
            3011 => WKBGeometryType::MultiCurveZM,
            3012 => WKBGeometryType::MultiSurfaceZM,
            3013 => WKBGeometryType::CurveZM,
            3014 => WKBGeometryType::SurfaceZM,
            3015 => WKBGeometryType::PolyhedralSurfaceZM,
            3016 => WKBGeometryType::TinZM,
            3017 => WKBGeometryType::TriangleZM,
            _ => WKBGeometryType::Unknown,
        }
    }
}

pub(crate) enum WKBByteOrder {
    Xdr = 0, // Big Endian
    Ndr = 1, // Little Endian
}

impl From<scroll::Endian> for WKBByteOrder {
    fn from(endian: scroll::Endian) -> Self {
        match endian {
            scroll::BE => WKBByteOrder::Xdr,
            scroll::LE => WKBByteOrder::Ndr,
        }
    }
}
