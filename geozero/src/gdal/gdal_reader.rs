use crate::error::{GeozeroError, Result};
use crate::{GeomProcessor, GeozeroGeometry};
use gdal::vector::Geometry;
use gdal_sys::{self, OGRwkbGeometryType};

impl GeozeroGeometry for Geometry {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        process_geom(self, processor)
    }
}

/// Process GDAL/OGR geometry.
pub fn process_geom<P: GeomProcessor>(geo: &Geometry, processor: &mut P) -> Result<()> {
    process_geom_n(geo, 0, processor)
}

fn process_geom_n<P: GeomProcessor>(geo: &Geometry, idx: usize, processor: &mut P) -> Result<()> {
    match type2d(geo.geometry_type()) {
        OGRwkbGeometryType::wkbPoint => {
            processor.point_begin(idx)?;
            process_point(geo, 0, processor)?;
            processor.point_end(idx)
        }
        OGRwkbGeometryType::wkbMultiPoint => {
            let n_pts = geo.geometry_count();
            processor.multipoint_begin(n_pts, idx)?;
            for i in 0..n_pts {
                let pt = unsafe { geo.get_unowned_geometry(i) };
                if type2d(pt.geometry_type()) != OGRwkbGeometryType::wkbPoint {
                    return Err(GeozeroError::GeometryFormat);
                }
                process_point(&pt, i, processor)?;
            }
            processor.multipoint_end(idx)
        }
        OGRwkbGeometryType::wkbLineString => process_linestring(geo, true, idx, processor),
        OGRwkbGeometryType::wkbMultiLineString => {
            let n_lines = geo.geometry_count();
            processor.multilinestring_begin(n_lines, idx)?;
            process_linestring_seq(geo, processor, n_lines)?;
            processor.multilinestring_end(idx)
        }
        OGRwkbGeometryType::wkbPolygon => process_polygon(geo, true, idx, processor),
        OGRwkbGeometryType::wkbMultiPolygon => {
            let n_polys = geo.geometry_count();
            processor.multipolygon_begin(n_polys, idx)?;
            for i in 0..n_polys {
                let poly = unsafe { geo.get_unowned_geometry(i) };
                if type2d(poly.geometry_type()) != OGRwkbGeometryType::wkbPolygon {
                    return Err(GeozeroError::GeometryFormat);
                }
                process_polygon(&poly, false, i, processor)?;
            }
            processor.multipolygon_end(idx)
        }
        OGRwkbGeometryType::wkbGeometryCollection => {
            let n_geoms = geo.geometry_count();
            processor.geometrycollection_begin(n_geoms, idx)?;
            for i in 0..n_geoms {
                let g = unsafe { geo.get_unowned_geometry(i) };
                process_geom_n(&g, i, processor)?;
            }
            processor.geometrycollection_end(idx)
        }
        OGRwkbGeometryType::wkbCircularString => process_circularstring(geo, idx, processor),
        OGRwkbGeometryType::wkbCompoundCurve => process_compoundcurve(geo, idx, processor),
        OGRwkbGeometryType::wkbCurvePolygon => process_curvepolygon(geo, idx, processor),
        OGRwkbGeometryType::wkbTriangle => process_triangle(geo, true, idx, processor),
        OGRwkbGeometryType::wkbMultiCurve => {
            let n_curves = geo.geometry_count();
            processor.multicurve_begin(n_curves, idx)?;
            for i in 0..n_curves {
                let geom = unsafe { geo.get_unowned_geometry(i) };
                process_curve(&geom, i, processor)?;
            }
            processor.multicurve_end(idx)
        }
        OGRwkbGeometryType::wkbPolyhedralSurface => {
            let n_polys = geo.geometry_count();
            processor.polyhedralsurface_begin(n_polys, idx)?;
            for i in 0..n_polys {
                let g = unsafe { geo.get_unowned_geometry(i) };
                process_polygon(&g, false, i, processor)?;
            }
            processor.polyhedralsurface_end(idx)
        }
        OGRwkbGeometryType::wkbTIN => {
            let n_triangles = geo.geometry_count();
            processor.tin_begin(n_triangles, idx)?;
            for i in 0..n_triangles {
                let g = unsafe { geo.get_unowned_geometry(i) };
                process_triangle(&g, false, i, processor)?;
            }
            processor.tin_end(idx)
        }
        OGRwkbGeometryType::wkbMultiSurface => {
            let n_polys = geo.geometry_count();
            processor.multisurface_begin(n_polys, idx)?;
            for i in 0..n_polys {
                let g = unsafe { geo.get_unowned_geometry(i) };
                let ty: OGRwkbGeometryType::Type = type2d(g.geometry_type());
                match ty {
                    OGRwkbGeometryType::wkbCurvePolygon => {
                        process_curvepolygon(&g, i, processor)?;
                    }
                    OGRwkbGeometryType::wkbPolygon => {
                        process_polygon(&g, false, i, processor)?;
                    }
                    _ => return Err(GeozeroError::GeometryFormat),
                }
            }
            processor.multisurface_end(idx)
        }
        _ => Err(GeozeroError::GeometryFormat),
    }
}

fn type2d(wkb_type: OGRwkbGeometryType::Type) -> OGRwkbGeometryType::Type {
    unsafe { gdal_sys::OGR_GT_Flatten(wkb_type) }
}

fn process_point<P: GeomProcessor>(geo: &Geometry, idx: usize, processor: &mut P) -> Result<()> {
    let multi = processor.dimensions().z;
    let (x, y, z) = geo.get_point(0);
    if multi {
        processor.coordinate(x, y, Some(z), None, None, None, idx)
    } else {
        processor.xy(x, y, idx)
    }
}

fn process_linestring_seq<P: GeomProcessor>(
    geo: &Geometry,
    processor: &mut P,
    geom_count: usize,
) -> Result<()> {
    for i in 0..geom_count {
        let geom = unsafe { geo.get_unowned_geometry(i) };
        if type2d(geom.geometry_type()) != OGRwkbGeometryType::wkbLineString {
            return Err(GeozeroError::GeometryFormat);
        }
        process_linestring(&geom, false, i, processor)?;
    }
    Ok(())
}

fn process_linestring<P: GeomProcessor>(
    geo: &Geometry,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let length = unsafe { gdal_sys::OGR_G_GetPointCount(geo.c_geometry()) } as usize;
    processor.linestring_begin(tagged, length, idx)?;
    let multi = processor.dimensions().z;
    for i in 0..length {
        let (x, y, z) = geo.get_point(i as i32);
        if multi {
            processor.coordinate(x, y, Some(z), None, None, None, i)?;
        } else {
            processor.xy(x, y, i)?;
        }
    }
    processor.linestring_end(tagged, idx)
}

fn process_circularstring<P: GeomProcessor>(
    geo: &Geometry,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let length = unsafe { gdal_sys::OGR_G_GetPointCount(geo.c_geometry()) } as usize;
    processor.circularstring_begin(length, idx)?;
    let multi = processor.dimensions().z;
    for i in 0..length {
        let (x, y, z) = geo.get_point(i as i32);
        if multi {
            processor.coordinate(x, y, Some(z), None, None, None, i)?;
        } else {
            processor.xy(x, y, i)?;
        }
    }
    processor.circularstring_end(idx)
}

fn process_polygon<P: GeomProcessor>(
    geo: &Geometry,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let ring_count = geo.geometry_count();
    processor.polygon_begin(tagged, ring_count, idx)?;
    process_linestring_seq(geo, processor, ring_count)?;
    processor.polygon_end(tagged, idx)
}

fn process_triangle<P: GeomProcessor>(
    geo: &Geometry,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let ring_count = geo.geometry_count();
    processor.triangle_begin(tagged, ring_count, idx)?;
    process_linestring_seq(geo, processor, ring_count)?;
    processor.triangle_end(tagged, idx)
}

fn process_compoundcurve<P: GeomProcessor>(
    geo: &Geometry,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let curve_count = geo.geometry_count();
    processor.compoundcurve_begin(curve_count, idx)?;
    for i in 0..curve_count {
        let geom = unsafe { geo.get_unowned_geometry(i) };
        let ty: OGRwkbGeometryType::Type = type2d(geom.geometry_type());
        match ty {
            OGRwkbGeometryType::wkbLineString => {
                process_linestring(&geom, false, i, processor)?;
            }
            OGRwkbGeometryType::wkbCircularString => {
                process_circularstring(&geom, i, processor)?;
            }
            _ => return Err(GeozeroError::GeometryFormat),
        }
    }
    processor.compoundcurve_end(idx)
}

fn process_curve<P: GeomProcessor>(geo: &Geometry, idx: usize, processor: &mut P) -> Result<()> {
    let ty: OGRwkbGeometryType::Type = type2d(geo.geometry_type());
    match ty {
        OGRwkbGeometryType::wkbLineString => process_linestring(geo, false, idx, processor),
        OGRwkbGeometryType::wkbCircularString => process_circularstring(geo, idx, processor),
        OGRwkbGeometryType::wkbCompoundCurve => process_compoundcurve(geo, idx, processor),
        _ => Err(GeozeroError::GeometryFormat),
    }
}

fn process_curvepolygon<P: GeomProcessor>(
    geo: &Geometry,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let ring_count = geo.geometry_count();
    processor.curvepolygon_begin(ring_count, idx)?;
    for i in 0..ring_count {
        let geom = unsafe { geo.get_unowned_geometry(i) };
        process_curve(&geom, i, processor)?;
    }
    processor.curvepolygon_end(idx)
}

#[cfg(test)]
#[cfg(all(feature = "with-wkt", feature = "with-wkb"))]
mod test {
    use super::*;
    use crate::wkt::WktWriter;
    use crate::{CoordDimensions, ToWkb, ToWkt};

    #[test]
    fn point() {
        let wkt = "POINT(1 1)";
        let geo = Geometry::from_wkt(wkt).unwrap();

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_geom(&geo, &mut WktWriter::new(&mut wkt_data)).is_ok());

        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), wkt);
    }

    #[test]
    fn multipoint() {
        let wkt = "MULTIPOINT(1 1,2 2)";
        let geo = Geometry::from_wkt(wkt).unwrap();
        assert_eq!(geo.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn line() {
        let wkt = "LINESTRING(1 1,2 2)";
        let geo = Geometry::from_wkt(wkt).unwrap();
        assert_eq!(geo.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn line_3d() {
        let wkt = "LINESTRING(1 1 10,2 2 20)";
        let geo = Geometry::from_wkt(wkt).unwrap();
        assert_eq!(
            geo.to_wkt_ndim(CoordDimensions {
                z: true,
                m: false,
                t: false,
                tm: false
            })
            .unwrap(),
            wkt
        );
    }

    #[test]
    fn multiline() {
        let wkt = "MULTILINESTRING((1 1,2 2),(3 3,4 4))";
        let geo = Geometry::from_wkt(wkt).unwrap();
        assert_eq!(geo.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn polygon() {
        let wkt = "POLYGON((0 0,0 3,3 3,3 0,0 0),(0.2 0.2,0.2 2,2 2,2 0.2,0.2 0.2))";
        let geo = Geometry::from_wkt(wkt).unwrap();
        assert_eq!(geo.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn multipolygon() {
        let wkt = "MULTIPOLYGON(((0 0,0 1,1 1,1 0,0 0)))";
        let geo = Geometry::from_wkt(wkt).unwrap();
        assert_eq!(geo.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn geometry_collection() {
        let wkt = "GEOMETRYCOLLECTION(POINT(1 1),LINESTRING(1 1,2 2))";
        let geo = Geometry::from_wkt(wkt).unwrap();
        assert_eq!(geo.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn circularstring() {
        // SELECT 'CIRCULARSTRING(0 0,1 1,2 0)'::geometry
        let wkb = hex::decode("01080000000300000000000000000000000000000000000000000000000000F03F000000000000F03F00000000000000400000000000000000").unwrap();
        let geo = Geometry::from_wkb(&wkb).unwrap();
        assert_eq!(geo.to_ewkb(CoordDimensions::default(), None).unwrap(), wkb);
    }

    #[test]
    fn compoundcurve() {
        // SELECT 'COMPOUNDCURVE (CIRCULARSTRING (0 0,1 1,2 0),(2 0,3 0))'::geometry
        let wkb = hex::decode("01090000000200000001080000000300000000000000000000000000000000000000000000000000F03F000000000000F03F000000000000004000000000000000000102000000020000000000000000000040000000000000000000000000000008400000000000000000").unwrap();
        let geo = Geometry::from_wkb(&wkb).unwrap();
        assert_eq!(geo.to_ewkb(CoordDimensions::default(), None).unwrap(), wkb);
    }

    #[test]
    fn curvepolygon() {
        // SELECT 'CURVEPOLYGON(COMPOUNDCURVE(CIRCULARSTRING(0 0,1 1,2 0),(2 0,3 0,3 -1,0 -1,0 0)))'::geometry
        let wkb = hex::decode("010A0000000100000001090000000200000001080000000300000000000000000000000000000000000000000000000000F03F000000000000F03F0000000000000040000000000000000001020000000500000000000000000000400000000000000000000000000000084000000000000000000000000000000840000000000000F0BF0000000000000000000000000000F0BF00000000000000000000000000000000").unwrap();
        let geo = Geometry::from_wkb(&wkb).unwrap();
        assert_eq!(geo.to_ewkb(CoordDimensions::default(), None).unwrap(), wkb);
    }

    #[test]
    fn multicurve() {
        // SELECT 'MULTICURVE((0 0, 5 5),CIRCULARSTRING(4 0, 4 4, 8 4))'::geometry
        let wkb = hex::decode("010B000000020000000102000000020000000000000000000000000000000000000000000000000014400000000000001440010800000003000000000000000000104000000000000000000000000000001040000000000000104000000000000020400000000000001040").unwrap();
        let geo = Geometry::from_wkb(&wkb).unwrap();
        assert_eq!(geo.to_ewkb(CoordDimensions::default(), None).unwrap(), wkb);
    }

    #[test]
    fn multisurface() {
        // SELECT 'MULTISURFACE (CURVEPOLYGON (COMPOUNDCURVE (CIRCULARSTRING (0 0,1 1,2 0),(2 0,3 0,3 -1,0 -1,0 0))))'::geometry
        let wkb = hex::decode("010C00000001000000010A0000000100000001090000000200000001080000000300000000000000000000000000000000000000000000000000F03F000000000000F03F0000000000000040000000000000000001020000000500000000000000000000400000000000000000000000000000084000000000000000000000000000000840000000000000F0BF0000000000000000000000000000F0BF00000000000000000000000000000000").unwrap();
        let geo = Geometry::from_wkb(&wkb).unwrap();
        assert_eq!(geo.to_ewkb(CoordDimensions::default(), None).unwrap(), wkb);
    }

    #[test]
    fn polyhedralsurface() {
        // SELECT 'POLYHEDRALSURFACE(((0 0 0,0 0 1,0 1 1,0 1 0,0 0 0)),((0 0 0,0 1 0,1 1 0,1 0 0,0 0 0)),((0 0 0,1 0 0,1 0 1,0 0 1,0 0 0)),((1 1 0,1 1 1,1 0 1,1 0 0,1 1 0)),((0 1 0,0 1 1,1 1 1,1 1 0,0 1 0)),((0 0 1,1 0 1,1 1 1,0 1 1,0 0 1)))'::geometry
        let wkb = hex::decode("010F000080060000000103000080010000000500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F000000000000F03F0000000000000000000000000000F03F0000000000000000000000000000000000000000000000000000000000000000010300008001000000050000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F000000000000F03F0000000000000000000000000000F03F0000000000000000000000000000000000000000000000000000000000000000000000000000000001030000800100000005000000000000000000000000000000000000000000000000000000000000000000F03F00000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F00000000000000000000000000000000000000000000F03F00000000000000000000000000000000000000000000000001030000800100000005000000000000000000F03F000000000000F03F0000000000000000000000000000F03F000000000000F03F000000000000F03F000000000000F03F0000000000000000000000000000F03F000000000000F03F00000000000000000000000000000000000000000000F03F000000000000F03F0000000000000000010300008001000000050000000000000000000000000000000000F03F00000000000000000000000000000000000000000000F03F000000000000F03F000000000000F03F000000000000F03F000000000000F03F000000000000F03F000000000000F03F00000000000000000000000000000000000000000000F03F00000000000000000103000080010000000500000000000000000000000000000000000000000000000000F03F000000000000F03F0000000000000000000000000000F03F000000000000F03F000000000000F03F000000000000F03F0000000000000000000000000000F03F000000000000F03F00000000000000000000000000000000000000000000F03F").unwrap();
        let geo = Geometry::from_wkb(&wkb).unwrap();
        dbg!(&geo);
        assert_eq!(geo.to_ewkb(CoordDimensions::xyz(), None).unwrap(), wkb);
    }

    #[test]
    fn tin() {
        // SELECT 'TIN(((0 0 0,0 0 1,0 1 0,0 0 0)),((0 0 0,0 1 0,1 1 0,0 0 0)))'::geometry
        let wkb = hex::decode("0110000080020000000111000080010000000400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F0000000000000000000000000000000000000000000000000000000000000000011100008001000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F000000000000F03F0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let geo = Geometry::from_wkb(&wkb).unwrap();
        dbg!(&geo);
        assert_eq!(geo.to_ewkb(CoordDimensions::xyz(), None).unwrap(), wkb);
    }

    #[test]
    fn triangle() {
        // SELECT 'TRIANGLE((0 0,0 9,9 0,0 0))'::geometry
        let wkb = hex::decode("0111000000010000000400000000000000000000000000000000000000000000000000000000000000000022400000000000002240000000000000000000000000000000000000000000000000").unwrap();
        let geo = Geometry::from_wkb(&wkb).unwrap();
        assert_eq!(geo.to_ewkb(CoordDimensions::default(), None).unwrap(), wkb);
    }
}
