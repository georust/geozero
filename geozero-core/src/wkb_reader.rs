use geozero::error::{GeozeroError, Result};
use geozero::GeomProcessor;
use scroll::IOread;
use std::io::Read;

/// WKB Types according to OGC 06-103r4 (https://www.ogc.org/standards/sfa)
#[derive(PartialEq, Debug)]
enum WKBGeometryType {
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
    // Extension to OGC spec
    Unknown = 255,
}

impl WKBGeometryType {
    fn from_u32(value: u32) -> Self {
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

#[allow(dead_code)]
enum WKBByteOrder {
    XDR = 0, // Big Endian
    NDR = 1, // Little Endian
}

#[derive(Debug)]
struct WkbInfo {
    endian: scroll::Endian,
    base_type: WKBGeometryType,
    has_z: bool,
    has_m: bool,
    srid: Option<i32>,
    envelope: Vec<f64>,
}

// OGC WKB header
fn read_wkb_header<R: Read>(raw: &mut R) -> Result<WkbInfo> {
    let byte_order = raw.ioread::<u8>()?;
    let endian = if byte_order == WKBByteOrder::XDR as u8 {
        scroll::BE
    } else {
        scroll::LE
    };
    let type_id = raw.ioread_with::<u32>(endian)?;
    let base_type = WKBGeometryType::from_u32(type_id % 1000);
    let type_id_dim = type_id / 1000;
    let has_z = type_id_dim == 1 || type_id_dim == 3;
    let has_m = type_id_dim == 2 || type_id_dim == 3;

    let info = WkbInfo {
        endian,
        base_type,
        has_z,
        has_m,
        srid: None,
        envelope: Vec::new(),
    };
    Ok(info)
}

/// EWKB header according to https://git.osgeo.org/gitea/postgis/postgis/src/branch/master/doc/ZMSgeoms.txt
fn read_ewkb_header<R: Read>(raw: &mut R) -> Result<WkbInfo> {
    let byte_order = raw.ioread::<u8>()?;
    let endian = if byte_order == WKBByteOrder::XDR as u8 {
        scroll::BE
    } else {
        scroll::LE
    };

    let type_id = raw.ioread_with::<u32>(endian)?;
    let base_type = WKBGeometryType::from_u32(type_id & 0xFF);
    let has_z = type_id & 0x80000000 == 0x80000000;
    let has_m = type_id & 0x40000000 == 0x40000000;

    let srid = if type_id & 0x20000000 == 0x20000000 {
        Some(raw.ioread_with::<i32>(endian)?)
    } else {
        None
    };

    let info = WkbInfo {
        endian,
        base_type,
        has_z,
        has_m,
        srid,
        envelope: Vec::new(),
    };
    Ok(info)
}

/// GPKG geometry header according to http://www.geopackage.org/spec/#gpb_format
fn read_gpkg_header<R: Read>(raw: &mut R) -> Result<WkbInfo> {
    let magic = [raw.ioread::<u8>()?, raw.ioread::<u8>()?];
    if &magic != b"GP" {
        return Err(GeozeroError::GeometryFormat);
    }
    let _version = raw.ioread::<u8>()?;
    let flags = raw.ioread::<u8>()?;
    // println!("flags: {:#010b}", flags);
    let _extended = (flags & 0b0010_0000) >> 5 == 1;
    let _empty = (flags & 0b0001_0000) >> 4 == 1;
    let env_len = match (flags & 0b0000_1110) >> 1 {
        0 => 0,
        1 => 4,
        2 => 6,
        3 => 6,
        4 => 8,
        _ => {
            return Err(GeozeroError::GeometryFormat);
        }
    };
    let endian = if flags & 0b0000_0001 == 0 {
        scroll::BE
    } else {
        scroll::LE
    };
    let srid = raw.ioread_with::<i32>(endian)?;
    let envelope: std::result::Result<Vec<f64>, _> = (0..env_len)
        .map(|_| raw.ioread_with::<f64>(endian))
        .collect();
    let envelope = envelope?;

    let ogc_info = read_wkb_header(raw)?;

    let info = WkbInfo {
        endian,
        base_type: ogc_info.base_type,
        has_z: ogc_info.has_z,
        has_m: ogc_info.has_m,
        srid: Some(srid),
        envelope,
    };
    Ok(info)
}

// TODO: Spatialite https://www.gaia-gis.it/gaia-sins/BLOB-Geometry.html

/// Process EWKB geometry
pub fn process_ewkb_geom<R: Read, P: GeomProcessor>(raw: &mut R, processor: &mut P) -> Result<()> {
    let info = read_ewkb_header(raw)?;
    process_wkb_geom_n(raw, &info, read_ewkb_header, 0, processor)
}

/// Process GPKG geometry
pub fn process_gpkg_geom<R: Read, P: GeomProcessor>(raw: &mut R, processor: &mut P) -> Result<()> {
    let info = read_gpkg_header(raw)?;
    process_wkb_geom_n(raw, &info, read_wkb_header, 0, processor)
}

fn process_wkb_geom_n<R: Read, P: GeomProcessor>(
    raw: &mut R,
    info: &WkbInfo,
    read_header: fn(&mut R) -> Result<WkbInfo>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    match info.base_type {
        WKBGeometryType::Point => {
            processor.point_begin(idx)?;
            process_coord(raw, &info, multi_dim(processor), 0, processor)?;
            processor.point_end(idx)?;
        }
        WKBGeometryType::MultiPoint => {
            let n_pts = raw.ioread_with::<u32>(info.endian)? as usize;
            processor.multipoint_begin(n_pts, idx)?;
            let multi = multi_dim(processor);
            for i in 0..n_pts {
                let info = read_header(raw)?;
                process_coord(raw, &info, multi, i, processor)?;
            }
            processor.multipoint_end(idx)?;
        }
        WKBGeometryType::LineString => {
            process_linestring(raw, &info, true, idx, processor)?;
        }
        WKBGeometryType::CircularString => {
            process_circularstring(raw, &info, idx, processor)?;
        }
        WKBGeometryType::CompoundCurve => {
            process_compoundcurve(raw, &info, read_header, idx, processor)?;
        }
        WKBGeometryType::MultiLineString => {
            let n_lines = raw.ioread_with::<u32>(info.endian)? as usize;
            processor.multilinestring_begin(n_lines, idx)?;
            for i in 0..n_lines {
                let info = read_header(raw)?;
                process_linestring(raw, &info, false, i, processor)?;
            }
            processor.multilinestring_end(idx)?;
        }
        WKBGeometryType::MultiCurve => {
            let n_curves = raw.ioread_with::<u32>(info.endian)? as usize;
            processor.multicurve_begin(n_curves, idx)?;
            for i in 0..n_curves {
                process_curve(raw, read_header, i, processor)?;
            }
            processor.multicurve_end(idx)?;
        }
        WKBGeometryType::Polygon => {
            process_polygon(raw, &info, true, idx, processor)?;
        }
        WKBGeometryType::Triangle => {
            let ring_count = raw.ioread_with::<u32>(info.endian)? as usize;
            processor.triangle_begin(ring_count, idx)?;
            for i in 0..ring_count {
                process_linestring(raw, info, false, i, processor)?;
            }
            processor.triangle_end(idx)?;
        }
        WKBGeometryType::CurvePolygon => {
            process_curvepolygon(raw, &info, read_header, idx, processor)?;
        }
        WKBGeometryType::MultiPolygon => {
            let n_polys = raw.ioread_with::<u32>(info.endian)? as usize;
            processor.multipolygon_begin(n_polys, idx)?;
            for i in 0..n_polys {
                let info = read_header(raw)?;
                process_polygon(raw, &info, false, i, processor)?;
            }
            processor.multipolygon_end(idx)?;
        }
        WKBGeometryType::PolyhedralSurface => {
            let n_polys = raw.ioread_with::<u32>(info.endian)? as usize;
            processor.polyhedralsurface_begin(n_polys, idx)?;
            for i in 0..n_polys {
                let info = read_header(raw)?;
                process_polygon(raw, &info, false, i, processor)?;
            }
            processor.polyhedralsurface_end(idx)?;
        }
        WKBGeometryType::Tin => {
            let n_polys = raw.ioread_with::<u32>(info.endian)? as usize;
            processor.tin_begin(n_polys, idx)?;
            for i in 0..n_polys {
                let info = read_header(raw)?;
                process_polygon(raw, &info, false, i, processor)?;
            }
            processor.tin_end(idx)?;
        }
        WKBGeometryType::MultiSurface => {
            let n_polys = raw.ioread_with::<u32>(info.endian)? as usize;
            processor.multisurface_begin(n_polys, idx)?;
            for i in 0..n_polys {
                let info = read_header(raw)?;
                match info.base_type {
                    WKBGeometryType::CurvePolygon => {
                        process_curvepolygon(raw, &info, read_header, i, processor)?;
                    }
                    WKBGeometryType::Polygon => {
                        process_polygon(raw, &info, false, i, processor)?;
                    }
                    _ => return Err(GeozeroError::GeometryFormat),
                }
            }
            processor.multisurface_end(idx)?;
        }

        WKBGeometryType::GeometryCollection => {
            let n_geoms = raw.ioread_with::<u32>(info.endian)? as usize;
            processor.geometrycollection_begin(n_geoms, idx)?;
            for i in 0..n_geoms {
                let info = read_header(raw)?;
                process_wkb_geom_n(raw, &info, read_header, i, processor)?;
            }
            processor.geometrycollection_end(idx)?;
        }
        _ => return Err(GeozeroError::GeometryFormat),
    }
    Ok(())
}

fn multi_dim<P: GeomProcessor>(processor: &P) -> bool {
    processor.dimensions().z || processor.dimensions().m
}

fn process_coord<R: Read, P: GeomProcessor>(
    raw: &mut R,
    info: &WkbInfo,
    multi_dim: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let x = raw.ioread_with::<f64>(info.endian)?;
    let y = raw.ioread_with::<f64>(info.endian)?;
    let z = if info.has_z {
        Some(raw.ioread_with::<f64>(info.endian)?)
    } else {
        None
    };
    let m = if info.has_m {
        Some(raw.ioread_with::<f64>(info.endian)?)
    } else {
        None
    };
    if multi_dim {
        processor.coordinate(x, y, z, m, None, None, idx)?;
    } else {
        processor.xy(x, y, idx)?;
    }
    Ok(())
}

fn process_linestring<R: Read, P: GeomProcessor>(
    raw: &mut R,
    info: &WkbInfo,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let length = raw.ioread_with::<u32>(info.endian)? as usize;
    processor.linestring_begin(tagged, length, idx)?;
    let multi = multi_dim(processor);
    for i in 0..length {
        process_coord(raw, info, multi, i, processor)?;
    }
    processor.linestring_end(tagged, idx)
}

fn process_circularstring<R: Read, P: GeomProcessor>(
    raw: &mut R,
    info: &WkbInfo,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let length = raw.ioread_with::<u32>(info.endian)? as usize;
    processor.circularstring_begin(length, idx)?;
    let multi = multi_dim(processor);
    for i in 0..length {
        process_coord(raw, info, multi, i, processor)?;
    }
    processor.circularstring_end(idx)
}

fn process_polygon<R: Read, P: GeomProcessor>(
    raw: &mut R,
    info: &WkbInfo,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let ring_count = raw.ioread_with::<u32>(info.endian)? as usize;
    processor.polygon_begin(tagged, ring_count, idx)?;
    for i in 0..ring_count {
        process_linestring(raw, info, false, i, processor)?;
    }
    processor.polygon_end(tagged, idx)
}

fn process_compoundcurve<R: Read, P: GeomProcessor>(
    raw: &mut R,
    info: &WkbInfo,
    read_header: fn(&mut R) -> Result<WkbInfo>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let n_strings = raw.ioread_with::<u32>(info.endian)? as usize;
    processor.compoundcurve_begin(n_strings, idx)?;
    for i in 0..n_strings {
        let info = read_header(raw)?;
        match info.base_type {
            WKBGeometryType::CircularString => {
                process_circularstring(raw, &info, i, processor)?;
            }
            WKBGeometryType::LineString => {
                process_linestring(raw, &info, false, i, processor)?;
            }
            _ => return Err(GeozeroError::GeometryFormat),
        }
    }
    processor.compoundcurve_end(idx)
}

fn process_curve<R: Read, P: GeomProcessor>(
    raw: &mut R,
    read_header: fn(&mut R) -> Result<WkbInfo>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let info = read_header(raw)?;
    match info.base_type {
        WKBGeometryType::CircularString => {
            process_circularstring(raw, &info, idx, processor)?;
        }
        WKBGeometryType::LineString => {
            process_linestring(raw, &info, false, idx, processor)?;
        }
        WKBGeometryType::CompoundCurve => {
            process_compoundcurve(raw, &info, read_header, idx, processor)?;
        }
        _ => return Err(GeozeroError::GeometryFormat),
    }
    Ok(())
}

fn process_curvepolygon<R: Read, P: GeomProcessor>(
    raw: &mut R,
    info: &WkbInfo,
    read_header: fn(&mut R) -> Result<WkbInfo>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let ring_count = raw.ioread_with::<u32>(info.endian)? as usize;
    processor.curvepolygon_begin(ring_count, idx)?;
    for i in 0..ring_count {
        process_curve(raw, read_header, i, processor)?;
    }
    processor.curvepolygon_end(idx)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::wkt_writer::WktWriter;

    #[test]
    fn ewkb_format() {
        // SELECT 'POINT(10 -20 100 1)'::geometry
        let ewkb = hex::decode(
            "01010000C0000000000000244000000000000034C00000000000005940000000000000F03F",
        )
        .unwrap();
        // Read header
        let info = read_ewkb_header(&mut ewkb.as_slice()).unwrap();
        assert_eq!(info.srid, None);
        assert!(info.has_z);
        assert!(info.has_m);

        // Process xy only
        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(
            process_ewkb_geom(&mut ewkb.as_slice(), &mut WktWriter::new(&mut wkt_data)).is_ok()
        );
        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), "POINT(10 -20)");

        // Process all dimensions
        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        writer.dims.z = true;
        writer.dims.m = true;
        assert!(process_ewkb_geom(&mut ewkb.as_slice(), &mut writer).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "POINT(10 -20 100 1)"
        );

        // SELECT 'SRID=4326;MULTIPOINT ((10 -20 100), (0 -0.5 101))'::geometry
        let ewkb = hex::decode("01040000A0E6100000020000000101000080000000000000244000000000000034C0000000000000594001010000800000000000000000000000000000E0BF0000000000405940").unwrap();

        // Read header
        let info = read_ewkb_header(&mut ewkb.as_slice()).unwrap();
        assert_eq!(info.base_type, WKBGeometryType::MultiPoint);
        assert_eq!(info.srid, Some(4326));
        assert!(info.has_z);

        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        writer.dims.z = true;
        assert!(process_ewkb_geom(&mut ewkb.as_slice(), &mut writer).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "MULTIPOINT(10 -20 100,0 -0.5 101)"
        );
    }

    #[test]
    fn ewkb_geometries() {
        // SELECT 'POINT(10 -20)'::geometry
        assert_eq!(
            &ewkb_to_wkt("0101000000000000000000244000000000000034C0", false),
            "POINT(10 -20)"
        );

        // SELECT 'SRID=4326;MULTIPOINT (10 -20 100, 0 -0.5 101)'::geometry
        assert_eq!(
            &ewkb_to_wkt("01040000A0E6100000020000000101000080000000000000244000000000000034C0000000000000594001010000800000000000000000000000000000E0BF0000000000405940", true),
            "MULTIPOINT(10 -20 100,0 -0.5 101)"
            //OGR: MULTIPOINT ((10 -20 100),(0 -0.5 101))
        );

        // SELECT 'SRID=4326;LINESTRING (10 -20 100, 0 -0.5 101)'::geometry
        assert_eq!(
            &ewkb_to_wkt("01020000A0E610000002000000000000000000244000000000000034C000000000000059400000000000000000000000000000E0BF0000000000405940", true),
            "LINESTRING(10 -20 100,0 -0.5 101)"
        );

        // SELECT 'SRID=4326;MULTILINESTRING ((10 -20, 0 -0.5), (0 0, 2 0))'::geometry
        assert_eq!(
            &ewkb_to_wkt("0105000020E610000002000000010200000002000000000000000000244000000000000034C00000000000000000000000000000E0BF0102000000020000000000000000000000000000000000000000000000000000400000000000000000", false),
            "MULTILINESTRING((10 -20,0 -0.5),(0 0,2 0))"
        );

        // SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry
        assert_eq!(
            &ewkb_to_wkt("0103000020E610000001000000050000000000000000000000000000000000000000000000000000400000000000000000000000000000004000000000000000400000000000000000000000000000004000000000000000000000000000000000", false),
            "POLYGON((0 0,2 0,2 2,0 2,0 0))"
        );

        // SELECT 'SRID=4326;MULTIPOLYGON (((0 0, 2 0, 2 2, 0 2, 0 0)), ((10 10, -2 10, -2 -2, 10 -2, 10 10)))'::geometry
        assert_eq!(
            &ewkb_to_wkt("0106000020E610000002000000010300000001000000050000000000000000000000000000000000000000000000000000400000000000000000000000000000004000000000000000400000000000000000000000000000004000000000000000000000000000000000010300000001000000050000000000000000002440000000000000244000000000000000C0000000000000244000000000000000C000000000000000C0000000000000244000000000000000C000000000000024400000000000002440", false),
            "MULTIPOLYGON(((0 0,2 0,2 2,0 2,0 0)),((10 10,-2 10,-2 -2,10 -2,10 10)))"
        );

        // SELECT 'GeometryCollection(POINT (10 10),POINT (30 30),LINESTRING (15 15, 20 20))'::geometry
        assert_eq!(
            &ewkb_to_wkt("01070000000300000001010000000000000000002440000000000000244001010000000000000000003E400000000000003E400102000000020000000000000000002E400000000000002E4000000000000034400000000000003440", false),
            "GEOMETRYCOLLECTION(POINT(10 10),POINT(30 30),LINESTRING(15 15,20 20))"
        );
    }

    #[test]
    fn ewkb_curves() {
        // SELECT 'CIRCULARSTRING(0 0,1 1,2 0)'::geometry
        assert_eq!(
            &ewkb_to_wkt("01080000000300000000000000000000000000000000000000000000000000F03F000000000000F03F00000000000000400000000000000000", false),
            "CIRCULARSTRING(0 0,1 1,2 0)"
        );

        // SELECT 'COMPOUNDCURVE (CIRCULARSTRING (0 0,1 1,2 0),(2 0,3 0))'::geometry
        assert_eq!(
            &ewkb_to_wkt("01090000000200000001080000000300000000000000000000000000000000000000000000000000F03F000000000000F03F000000000000004000000000000000000102000000020000000000000000000040000000000000000000000000000008400000000000000000", false),
            "COMPOUNDCURVE(CIRCULARSTRING(0 0,1 1,2 0),(2 0,3 0))"
        );

        // SELECT 'CURVEPOLYGON(COMPOUNDCURVE(CIRCULARSTRING(0 0,1 1,2 0),(2 0,3 0,3 -1,0 -1,0 0)))'::geometry
        assert_eq!(
            &ewkb_to_wkt("010A0000000100000001090000000200000001080000000300000000000000000000000000000000000000000000000000F03F000000000000F03F0000000000000040000000000000000001020000000500000000000000000000400000000000000000000000000000084000000000000000000000000000000840000000000000F0BF0000000000000000000000000000F0BF00000000000000000000000000000000", false),
            "CURVEPOLYGON(COMPOUNDCURVE(CIRCULARSTRING(0 0,1 1,2 0),(2 0,3 0,3 -1,0 -1,0 0)))"
        );

        // SELECT 'MULTICURVE((0 0, 5 5),CIRCULARSTRING(4 0, 4 4, 8 4))'::geometry
        assert_eq!(
            &ewkb_to_wkt("010B000000020000000102000000020000000000000000000000000000000000000000000000000014400000000000001440010800000003000000000000000000104000000000000000000000000000001040000000000000104000000000000020400000000000001040", false),
            "MULTICURVE((0 0,5 5),CIRCULARSTRING(4 0,4 4,8 4))"
        );

        // SELECT 'MULTISURFACE (CURVEPOLYGON (COMPOUNDCURVE (CIRCULARSTRING (0 0,1 1,2 0),(2 0,3 0,3 -1,0 -1,0 0))))'::geometry
        assert_eq!(
            &ewkb_to_wkt("010C00000001000000010A0000000100000001090000000200000001080000000300000000000000000000000000000000000000000000000000F03F000000000000F03F0000000000000040000000000000000001020000000500000000000000000000400000000000000000000000000000084000000000000000000000000000000840000000000000F0BF0000000000000000000000000000F0BF00000000000000000000000000000000", false),
            "MULTISURFACE(CURVEPOLYGON(COMPOUNDCURVE(CIRCULARSTRING(0 0,1 1,2 0),(2 0,3 0,3 -1,0 -1,0 0))))"
        );
    }

    #[test]
    fn ewkb_surfaces() {
        // SELECT 'POLYHEDRALSURFACE(((0 0 0,0 0 1,0 1 1,0 1 0,0 0 0)),((0 0 0,0 1 0,1 1 0,1 0 0,0 0 0)),((0 0 0,1 0 0,1 0 1,0 0 1,0 0 0)),((1 1 0,1 1 1,1 0 1,1 0 0,1 1 0)),((0 1 0,0 1 1,1 1 1,1 1 0,0 1 0)),((0 0 1,1 0 1,1 1 1,0 1 1,0 0 1)))'::geometry
        assert_eq!(
            &ewkb_to_wkt("010F000080060000000103000080010000000500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F000000000000F03F0000000000000000000000000000F03F0000000000000000000000000000000000000000000000000000000000000000010300008001000000050000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F000000000000F03F0000000000000000000000000000F03F0000000000000000000000000000000000000000000000000000000000000000000000000000000001030000800100000005000000000000000000000000000000000000000000000000000000000000000000F03F00000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F00000000000000000000000000000000000000000000F03F00000000000000000000000000000000000000000000000001030000800100000005000000000000000000F03F000000000000F03F0000000000000000000000000000F03F000000000000F03F000000000000F03F000000000000F03F0000000000000000000000000000F03F000000000000F03F00000000000000000000000000000000000000000000F03F000000000000F03F0000000000000000010300008001000000050000000000000000000000000000000000F03F00000000000000000000000000000000000000000000F03F000000000000F03F000000000000F03F000000000000F03F000000000000F03F000000000000F03F000000000000F03F00000000000000000000000000000000000000000000F03F00000000000000000103000080010000000500000000000000000000000000000000000000000000000000F03F000000000000F03F0000000000000000000000000000F03F000000000000F03F000000000000F03F000000000000F03F0000000000000000000000000000F03F000000000000F03F00000000000000000000000000000000000000000000F03F", true),
            "POLYHEDRALSURFACE(((0 0 0,0 0 1,0 1 1,0 1 0,0 0 0)),((0 0 0,0 1 0,1 1 0,1 0 0,0 0 0)),((0 0 0,1 0 0,1 0 1,0 0 1,0 0 0)),((1 1 0,1 1 1,1 0 1,1 0 0,1 1 0)),((0 1 0,0 1 1,1 1 1,1 1 0,0 1 0)),((0 0 1,1 0 1,1 1 1,0 1 1,0 0 1)))"
        );
        // SELECT 'TIN(((0 0 0,0 0 1,0 1 0,0 0 0)),((0 0 0,0 1 0,1 1 0,0 0 0)))'::geometry
        assert_eq!(
            &ewkb_to_wkt("0110000080020000000111000080010000000400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F0000000000000000000000000000000000000000000000000000000000000000011100008001000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F000000000000F03F0000000000000000000000000000000000000000000000000000000000000000", true),
            "TIN(((0 0 0,0 0 1,0 1 0,0 0 0)),((0 0 0,0 1 0,1 1 0,0 0 0)))"
        );

        // SELECT 'TRIANGLE((0 0,0 9,9 0,0 0))'::geometry
        assert_eq!(
            &ewkb_to_wkt("0111000000010000000400000000000000000000000000000000000000000000000000000000000000000022400000000000002240000000000000000000000000000000000000000000000000", false),
            "TRIANGLE((0 0,0 9,9 0,0 0))"
        );
    }

    fn ewkb_to_wkt(ewkbstr: &str, with_z: bool) -> String {
        let ewkb = hex::decode(ewkbstr).unwrap();
        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        writer.dims.z = with_z;
        assert_eq!(
            process_ewkb_geom(&mut ewkb.as_slice(), &mut writer).map_err(|e| e.to_string()),
            Ok(())
        );
        std::str::from_utf8(&wkt_data).unwrap().to_string()
    }

    #[test]
    fn gpkg_geometries() {
        // pt2d
        let wkb = hex::decode("47500003E61000009A9999999999F13F9A9999999999F13F9A9999999999F13F9A9999999999F13F01010000009A9999999999F13F9A9999999999F13F").unwrap();
        let info = read_gpkg_header(&mut wkb.as_slice()).unwrap();
        assert_eq!(info.base_type, WKBGeometryType::Point);
        assert!(!info.has_z);
        assert!(!info.has_m);
        assert_eq!(info.srid, Some(4326));

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_gpkg_geom(&mut wkb.as_slice(), &mut WktWriter::new(&mut wkt_data)).is_ok());
        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), "POINT(1.1 1.1)");

        // mln3dzm
        let wkb = hex::decode("47500003E6100000000000000000244000000000000034400000000000002440000000000000344001BD0B00000100000001BA0B0000020000000000000000003440000000000000244000000000000008400000000000001440000000000000244000000000000034400000000000001C400000000000000040").unwrap();
        let info = read_gpkg_header(&mut wkb.as_slice()).unwrap();
        assert_eq!(info.base_type, WKBGeometryType::MultiLineString);
        assert!(info.has_z);
        assert!(info.has_m);
        assert_eq!(info.envelope, vec![10.0, 20.0, 10.0, 20.0]);

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_gpkg_geom(&mut wkb.as_slice(), &mut WktWriter::new(&mut wkt_data)).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "MULTILINESTRING((20 10,10 20))"
        );

        // gc2d
        let wkb = hex::decode("47500003e6100000000000000000f03f0000000000003640000000000000084000000000000036400107000000020000000101000000000000000000f03f00000000000008400103000000010000000400000000000000000035400000000000003540000000000000364000000000000035400000000000003540000000000000364000000000000035400000000000003540").unwrap();
        let info = read_gpkg_header(&mut wkb.as_slice()).unwrap();
        assert_eq!(info.base_type, WKBGeometryType::GeometryCollection);
        assert_eq!(info.envelope, vec![1.0, 22.0, 3.0, 22.0]);

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_gpkg_geom(&mut wkb.as_slice(), &mut WktWriter::new(&mut wkt_data)).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "GEOMETRYCOLLECTION(POINT(1 3),POLYGON((21 21,22 21,21 22,21 21)))"
        );
    }

    #[test]
    fn scroll_error() {
        let err = read_ewkb_header(&mut std::io::Cursor::new(b"")).unwrap_err();
        assert_eq!(err.to_string(), "I/O error");
    }
}
