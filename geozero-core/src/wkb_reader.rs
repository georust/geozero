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
    Triangle = 17,
    MultiPoint = 4,
    MultiLineString = 5,
    MultiPolygon = 6,
    GeometryCollection = 7,
    PolyhedralSurface = 15,
    TIN = 16,
    PointZ = 1001,
    LineStringZ = 1002,
    PolygonZ = 1003,
    Trianglez = 1017,
    MultiPointZ = 1004,
    MultiLineStringZ = 1005,
    MultiPolygonZ = 1006,
    GeometryCollectionZ = 1007,
    PolyhedralSurfaceZ = 1015,
    TINZ = 1016,
    PointM = 2001,
    LineStringM = 2002,
    PolygonM = 2003,
    TriangleM = 2017,
    MultiPointM = 2004,
    MultiLineStringM = 2005,
    MultiPolygonM = 2006,
    GeometryCollectionM = 2007,
    PolyhedralSurfaceM = 2015,
    TINM = 2016,
    PointZM = 3001,
    LineStringZM = 3002,
    PolygonZM = 3003,
    TriangleZM = 3017,
    MultiPointZM = 3004,
    MultiLineStringZM = 3005,
    MultiPolygonZM = 3006,
    GeometryCollectionZM = 3007,
    PolyhedralSurfaceZM = 3015,
    TinZM = 3016,
    // Extension to OGC spec
    Unknown = 255,
}

impl WKBGeometryType {
    fn from_u32(value: u32) -> Self {
        match value {
            1 => WKBGeometryType::Point,
            2 => WKBGeometryType::LineString,
            3 => WKBGeometryType::Polygon,
            17 => WKBGeometryType::Triangle,
            4 => WKBGeometryType::MultiPoint,
            5 => WKBGeometryType::MultiLineString,
            6 => WKBGeometryType::MultiPolygon,
            7 => WKBGeometryType::GeometryCollection,
            15 => WKBGeometryType::PolyhedralSurface,
            16 => WKBGeometryType::TIN,
            1001 => WKBGeometryType::PointZ,
            1002 => WKBGeometryType::LineStringZ,
            1003 => WKBGeometryType::PolygonZ,
            1017 => WKBGeometryType::Trianglez,
            1004 => WKBGeometryType::MultiPointZ,
            1005 => WKBGeometryType::MultiLineStringZ,
            1006 => WKBGeometryType::MultiPolygonZ,
            1007 => WKBGeometryType::GeometryCollectionZ,
            1015 => WKBGeometryType::PolyhedralSurfaceZ,
            1016 => WKBGeometryType::TINZ,
            2001 => WKBGeometryType::PointM,
            2002 => WKBGeometryType::LineStringM,
            2003 => WKBGeometryType::PolygonM,
            2017 => WKBGeometryType::TriangleM,
            2004 => WKBGeometryType::MultiPointM,
            2005 => WKBGeometryType::MultiLineStringM,
            2006 => WKBGeometryType::MultiPolygonM,
            2007 => WKBGeometryType::GeometryCollectionM,
            2015 => WKBGeometryType::PolyhedralSurfaceM,
            2016 => WKBGeometryType::TINM,
            3001 => WKBGeometryType::PointZM,
            3002 => WKBGeometryType::LineStringZM,
            3003 => WKBGeometryType::PolygonZM,
            3017 => WKBGeometryType::TriangleZM,
            3004 => WKBGeometryType::MultiPointZM,
            3005 => WKBGeometryType::MultiLineStringZM,
            3006 => WKBGeometryType::MultiPolygonZM,
            3007 => WKBGeometryType::GeometryCollectionZM,
            3015 => WKBGeometryType::PolyhedralSurfaceZM,
            3016 => WKBGeometryType::TinZM,
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
    // envelope: Vec<f64>,
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
    };
    Ok(info)
}

// TODO: GPKG http://www.geopackage.org/spec/#gpb_format
// TODO: Spatialite https://www.gaia-gis.it/gaia-sins/BLOB-Geometry.html

/// Process EWKB geometry
pub fn process_wkb_geom<R: Read, P: GeomProcessor>(raw: &mut R, processor: &mut P) -> Result<()> {
    process_wkb_geom_n(raw, 0, processor)
}

fn process_wkb_geom_n<R: Read, P: GeomProcessor>(
    raw: &mut R,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let info = read_ewkb_header(raw)?;
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
                let info = read_ewkb_header(raw)?;
                process_coord(raw, &info, multi, i, processor)?;
            }
            processor.multipoint_end(idx)?;
        }
        WKBGeometryType::LineString => {
            process_linestring(raw, &info, true, 0, processor)?;
        }
        WKBGeometryType::MultiLineString => {
            let n_lines = raw.ioread_with::<u32>(info.endian)? as usize;
            processor.multilinestring_begin(n_lines, idx)?;
            for i in 0..n_lines {
                let info = read_ewkb_header(raw)?;
                process_linestring(raw, &info, false, i, processor)?;
            }
            processor.multilinestring_end(idx)?;
        }
        WKBGeometryType::Polygon => {
            process_polygon(raw, &info, true, 0, processor)?;
        }
        WKBGeometryType::MultiPolygon => {
            let n_polys = raw.ioread_with::<u32>(info.endian)? as usize;
            processor.multipolygon_begin(n_polys, idx)?;
            for i in 0..n_polys {
                let info = read_ewkb_header(raw)?;
                process_polygon(raw, &info, false, i, processor)?;
            }
            processor.multipolygon_end(idx)?;
        }
        WKBGeometryType::GeometryCollection => {
            let n_geoms = raw.ioread_with::<u32>(info.endian)? as usize;
            for i in 0..n_geoms {
                process_wkb_geom_n(raw, i, processor)?;
            }
            return Err(GeozeroError::GeometryFormat); //TODO
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::wkt_writer::WktWriter;

    fn hex_to_vec(hexstr: &str) -> Vec<u8> {
        hexstr
            .as_bytes()
            .chunks(2)
            .map(|chars| {
                let hb = if chars[0] <= 57 {
                    chars[0] - 48
                } else {
                    chars[0] - 55
                };
                let lb = if chars[1] <= 57 {
                    chars[1] - 48
                } else {
                    chars[1] - 55
                };
                hb * 16 + lb
            })
            .collect()
    }

    #[test]
    fn ewkb_geometries() {
        // SELECT 'POINT(10 -20 100 1)'::geometry
        let ewkb = hex_to_vec(
            "01010000C0000000000000244000000000000034C00000000000005940000000000000F03F",
        );
        // Read header
        let info = read_ewkb_header(&mut ewkb.as_slice()).unwrap();
        assert_eq!(info.srid, None);
        assert!(info.has_z);
        assert!(info.has_m);

        // Process xy only
        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_wkb_geom(&mut ewkb.as_slice(), &mut WktWriter::new(&mut wkt_data)).is_ok());
        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), "POINT (10 -20)");

        // Process all dimensions
        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        writer.dims.z = true;
        writer.dims.m = true;
        assert!(process_wkb_geom(&mut ewkb.as_slice(), &mut writer).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "POINT (10 -20 100 1)"
        );

        // SELECT 'SRID=4326;MULTIPOINT ((10 -20 100), (0 -0.5 101))'::geometry
        let ewkb = hex_to_vec("01040000A0E6100000020000000101000080000000000000244000000000000034C0000000000000594001010000800000000000000000000000000000E0BF0000000000405940");

        // Read header
        let info = read_ewkb_header(&mut ewkb.as_slice()).unwrap();
        assert_eq!(info.base_type, WKBGeometryType::MultiPoint);
        assert_eq!(info.srid, Some(4326));
        assert!(info.has_z);

        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        writer.dims.z = true;
        assert!(process_wkb_geom(&mut ewkb.as_slice(), &mut writer).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "MULTIPOINT (10 -20 100, 0 -0.5 101)"
        );

        // SELECT 'SRID=4326;LINESTRING (10 -20 100, 0 -0.5 101)'::geometry
        let ewkb = hex_to_vec("01020000A0E610000002000000000000000000244000000000000034C000000000000059400000000000000000000000000000E0BF0000000000405940");
        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        writer.dims.z = true;
        assert!(process_wkb_geom(&mut ewkb.as_slice(), &mut writer).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "LINESTRING (10 -20 100, 0 -0.5 101)"
        );

        // SELECT 'SRID=4326;MULTILINESTRING ((10 -20, 0 -0.5), (0 0, 2 0))'::geometry
        let ewkb = hex_to_vec("0105000020E610000002000000010200000002000000000000000000244000000000000034C00000000000000000000000000000E0BF0102000000020000000000000000000000000000000000000000000000000000400000000000000000");
        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        writer.dims.z = true;
        assert!(process_wkb_geom(&mut ewkb.as_slice(), &mut writer).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "MULTILINESTRING ((10 -20, 0 -0.5), (0 0, 2 0))"
        );

        // SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry
        let ewkb = hex_to_vec("0103000020E610000001000000050000000000000000000000000000000000000000000000000000400000000000000000000000000000004000000000000000400000000000000000000000000000004000000000000000000000000000000000");
        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        assert!(process_wkb_geom(&mut ewkb.as_slice(), &mut writer).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))"
        );

        // SELECT 'SRID=4326;MULTIPOLYGON (((0 0, 2 0, 2 2, 0 2, 0 0)), ((10 10, -2 10, -2 -2, 10 -2, 10 10)))'::geometry
        let ewkb = hex_to_vec("0106000020E610000002000000010300000001000000050000000000000000000000000000000000000000000000000000400000000000000000000000000000004000000000000000400000000000000000000000000000004000000000000000000000000000000000010300000001000000050000000000000000002440000000000000244000000000000000C0000000000000244000000000000000C000000000000000C0000000000000244000000000000000C000000000000024400000000000002440");
        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        assert!(process_wkb_geom(&mut ewkb.as_slice(), &mut writer).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "MULTIPOLYGON (((0 0, 2 0, 2 2, 0 2, 0 0)), ((10 10, -2 10, -2 -2, 10 -2, 10 10)))"
        );

        // SELECT 'GeometryCollection(POINT (10 10),POINT (30 30),LINESTRING (15 15, 20 20))'::geometry
        let ewkb = hex_to_vec("01070000000300000001010000000000000000002440000000000000244001010000000000000000003E400000000000003E400102000000020000000000000000002E400000000000002E4000000000000034400000000000003440");
        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        assert!(process_wkb_geom(&mut ewkb.as_slice(), &mut writer).is_err());
        // assert!(process_wkb_geom(&mut ewkb.as_slice(), &mut writer).is_ok());
        // assert_eq!(
        //     std::str::from_utf8(&wkt_data).unwrap(),
        //     "POINT (10 10)POINT (30 30)LINESTRING (15 15, 20 20)"
        // );
    }

    #[test]
    fn scroll_error() {
        let err = read_ewkb_header(&mut std::io::Cursor::new(b"")).unwrap_err();
        assert_eq!(err.to_string(), "I/O error");
    }
}
