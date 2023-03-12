use crate::error::{GeozeroError, Result};
use crate::{FeatureProcessor, GeomProcessor, GeozeroDatasource, GeozeroGeometry};

use std::io::Read;
use wkt::types::{
    Coord, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon,
};
use wkt::Geometry;

/// WKT String.
#[derive(Debug)]
pub struct WktString(pub String);

impl GeozeroGeometry for WktString {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        read_wkt(&mut self.0.as_bytes(), processor)
    }
}

/// WKT String slice.
pub struct WktStr<'a>(pub &'a str);

impl GeozeroGeometry for WktStr<'_> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        read_wkt(&mut self.0.as_bytes(), processor)
    }
}

impl GeozeroDatasource for WktStr<'_> {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        read_wkt(&mut self.0.as_bytes(), processor)
    }
}

/// Wkt Reader.
pub struct WktReader<'a, R: Read>(pub &'a mut R);

impl<'a, R: Read> GeozeroDatasource for WktReader<'a, R> {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        read_wkt(&mut self.0, processor)
    }
}

/// Read and process WKT geometry.
pub fn read_wkt<R: Read, P: GeomProcessor>(reader: &mut R, processor: &mut P) -> Result<()> {
    use std::str::FromStr;
    // PERF: it would be good to avoid copying data into this string when we already
    // have a string as input. Maybe the wkt crate needs a from_reader implementation.
    let mut wkt_string = String::new();
    reader.read_to_string(&mut wkt_string)?;
    let wkt = wkt::Wkt::from_str(&wkt_string).map_err(|e| GeozeroError::Geometry(e.to_string()))?;
    process_wkt_geom(&wkt.item, processor)
}

/// Process WKT geometry
fn process_wkt_geom<P: GeomProcessor>(geometry: &Geometry<f64>, processor: &mut P) -> Result<()> {
    process_wkt_geom_n(geometry, 0, processor)
}

pub(crate) fn process_wkt_geom_n<P: GeomProcessor>(
    geometry: &Geometry<f64>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let multi_dim = processor.multi_dim();
    match geometry {
        Geometry::Point(g) => process_point(g, multi_dim, idx, processor)?,
        Geometry::MultiPoint(g) => process_multi_point(g, idx, processor)?,
        Geometry::LineString(g) => process_linestring(g, true, idx, processor)?,
        Geometry::MultiLineString(g) => process_multilinestring(g, idx, processor)?,
        Geometry::Polygon(g) => process_polygon(g, true, idx, processor)?,
        Geometry::MultiPolygon(g) => process_multi_polygon(g, idx, processor)?,
        Geometry::GeometryCollection(g) => process_geometry_collection(g, idx, processor)?,
    }
    Ok(())
}

fn process_geometry_collection<P: GeomProcessor>(
    geometry_collection: &GeometryCollection<f64>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.geometrycollection_begin(geometry_collection.0.len(), idx)?;
    for (idxg, geometry) in geometry_collection.0.iter().enumerate() {
        process_wkt_geom_n(geometry, idxg, processor)?;
    }
    processor.geometrycollection_end(idx)
}

fn process_coord<P: GeomProcessor>(
    coord: &Coord<f64>,
    multi_dim: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    if multi_dim {
        processor.coordinate(coord.x, coord.y, coord.z, coord.m, None, None, idx)
    } else {
        processor.xy(coord.x, coord.y, idx)
    }
}

fn process_point<P: GeomProcessor>(
    point: &Point<f64>,
    multi_dim: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    if let Some(ref coord) = point.0 {
        processor.point_begin(idx)?;
        process_coord(coord, multi_dim, 0, processor)?;
        processor.point_end(idx)
    } else {
        processor.empty_point(idx)
    }
}

fn process_multi_point<P: GeomProcessor>(
    multi_point: &MultiPoint<f64>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.multipoint_begin(multi_point.0.len(), idx)?;
    let multi_dim = processor.multi_dim();
    for (idxc, point) in multi_point.0.iter().enumerate() {
        if let Some(ref coord) = point.0 {
            process_coord(coord, multi_dim, idxc, processor)?;
        } else {
            // skip processing of the untagged empty POINT, since no other formats support it.
            // Alternatively we could error here, but likely omitting the empty coord won't affect
            // the output of most computations (area, length, etc.)
        }
    }
    processor.multipoint_end(idx)
}

fn process_linestring<P: GeomProcessor>(
    linestring: &LineString<f64>,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.linestring_begin(tagged, linestring.0.len(), idx)?;
    let multi_dim = processor.multi_dim();
    for (idxc, coord) in linestring.0.iter().enumerate() {
        process_coord(coord, multi_dim, idxc, processor)?
    }
    processor.linestring_end(tagged, idx)
}

fn process_multilinestring<P: GeomProcessor>(
    multilinestring: &MultiLineString<f64>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.multilinestring_begin(multilinestring.0.len(), idx)?;
    for (idxc, linestring) in multilinestring.0.iter().enumerate() {
        process_linestring(linestring, false, idxc, processor)?
    }
    processor.multilinestring_end(idx)
}

fn process_polygon<P: GeomProcessor>(
    polygon: &Polygon<f64>,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.polygon_begin(tagged, polygon.0.len(), idx)?;
    for (idxl, linestring_type) in polygon.0.iter().enumerate() {
        process_linestring(linestring_type, false, idxl, processor)?
    }
    processor.polygon_end(tagged, idx)
}

fn process_multi_polygon<P: GeomProcessor>(
    multi_polygon: &MultiPolygon<f64>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.multipolygon_begin(multi_polygon.0.len(), idx)?;
    for (idxp, polygon) in multi_polygon.0.iter().enumerate() {
        process_polygon(polygon, false, idxp, processor)?;
    }
    processor.multipolygon_end(idx)
}

#[cfg(all(test, feature = "with-geo"))]
mod test {
    use super::*;
    use crate::geo_types::conversion::ToGeo;
    use crate::ToWkt;
    use geo_types::{line_string, point, polygon};

    #[test]
    fn point() {
        let wkt = WktStr("POINT(1.0 2.0)");
        let actual = wkt.to_geo().unwrap();

        let expected: geo_types::Geometry<f64> = point!(x: 1.0, y: 2.0).into();

        assert_eq!(expected, actual);
    }

    #[test]
    fn multi_point() {
        // Both of these are failing
        let wkt_1 = WktStr("MULTIPOINT ((10 40), (40 30), (20 20), (30 10))");
        let actual_1 = wkt_1.to_geo().unwrap();

        // alternative spelling
        let wkt_2 = WktStr("MULTIPOINT (10 40, 40 30, 20 20, 30 10)");
        let actual_2 = wkt_2.to_geo().unwrap();

        let expected: geo_types::Geometry<f64> = geo_types::MultiPoint(vec![
            point!(x: 10.0, y: 40.0),
            point!(x: 40.0, y: 30.0),
            point!(x: 20.0, y: 20.0),
            point!(x: 30.0, y: 10.0),
        ])
        .into();

        assert_eq!(expected, actual_1);
        assert_eq!(expected, actual_2);
    }

    #[test]
    fn line_string() {
        let wkt = WktStr("LINESTRING (30 10, 10 30, 40 40)");
        let actual = wkt.to_geo().unwrap();

        let expected: geo_types::Geometry<f64> =
            line_string![(x: 30.0, y: 10.0), (x: 10.0, y: 30.0), (x: 40.0, y: 40.0)].into();
        assert_eq!(expected, actual);
    }

    #[test]
    fn multi_line_string() {
        let wkt = WktStr("MULTILINESTRING ((10 10, 20 20, 10 40), (40 40, 30 30, 40 20, 30 10))");
        let actual = wkt.to_geo().unwrap();
        // type one line at a time.
        let expected: geo_types::Geometry<f64> = geo_types::MultiLineString(vec![
            line_string![(x: 10.0, y: 10.0), (x: 20.0, y: 20.0), (x: 10.0, y: 40.0)],
            line_string![(x: 40.0, y: 40.0), (x: 30.0, y: 30.0), (x: 40.0, y: 20.0), (x: 30.0, y: 10.0)],
        ]).into();
        assert_eq!(expected, actual);
    }

    #[test]
    fn polygon() {
        let wkt = WktStr("POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))");
        let actual = wkt.to_geo().unwrap();

        let expected: geo_types::Geometry<f64> = polygon![(x: 30.0, y: 10.0), (x: 40.0, y: 40.0), (x: 20.0, y: 40.0), (x: 10.0, y: 20.0), (x: 30.0, y: 10.0)].into();
        assert_eq!(expected, actual);
    }

    #[test]
    fn polygon_with_hole() {
        let wkt =
            WktStr("POLYGON ((35 10, 45 45, 15 40, 10 20, 35 10), (20 30, 35 35, 30 20, 20 30))");
        let actual = wkt.to_geo().unwrap();

        let expected: geo_types::Geometry<f64> = polygon!(
            exterior: [(x: 35.0,  y: 10.0), (x: 45.0, y: 45.0), (x: 15.0, y: 40.0), (x: 10.0, y: 20.0), (x: 35.0, y: 10.0)],
            interiors: [
                [(x: 20.0, y: 30.0), (x: 35.0, y: 35.0), (x: 30.0, y: 20.0), (x: 20.0, y: 30.0)]
            ]).into();
        assert_eq!(expected, actual);
    }

    #[test]
    fn multi_polygon() {
        let wkt = WktStr(
            "MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)),
            ((15 5, 40 10, 10 20, 5 10, 15 5)))",
        );
        let actual = wkt.to_geo().unwrap();

        let expected: geo_types::Geometry<f64> = geo_types::MultiPolygon(vec![
            polygon![(x: 30.0, y: 20.0), (x: 45.0, y: 40.0), (x: 10.0, y: 40.0), (x: 30.0, y: 20.0)],
            polygon![(x: 15.0, y: 5.0), (x: 40.0, y: 10.0), (x: 10.0, y: 20.0), (x: 5.0, y: 10.0), (x: 15.0, y: 5.0)],
        ]).into();
        assert_eq!(expected, actual);
    }

    #[test]
    fn multi_polygon_with_holes() {
        let wkt = WktStr(
            "MULTIPOLYGON (((40 40, 20 45, 45 30, 40 40)),
                         ((35 10, 45 45, 15 40, 10 20, 35 10),
                         (20 30, 35 35, 30 20, 20 30)))",
        );
        let actual = wkt.to_geo().unwrap();

        let expected: geo_types::Geometry<f64> = geo_types::MultiPolygon(vec![
            polygon![(x: 40.0, y: 40.0), (x: 20.0, y: 45.0), (x: 45.0, y: 30.0), (x: 40.0, y: 40.0)],
            polygon!(
            exterior: [(x: 35.0,  y: 10.0), (x: 45.0, y: 45.0), (x: 15.0, y: 40.0), (x: 10.0, y: 20.0), (x: 35.0, y: 10.0)],
            interiors: [
                [(x: 20.0, y: 30.0), (x: 35.0, y: 35.0), (x: 30.0, y: 20.0), (x: 20.0, y: 30.0)]
            ])
        ]).into();

        assert_eq!(expected, actual);
    }

    #[test]
    fn geometry_collection() {
        let wkt = WktStr(
            "GEOMETRYCOLLECTION (POINT (40 10),
                        LINESTRING (10 10, 20 20, 10 40),
                        POLYGON ((40 40, 20 45, 45 30, 40 40)))",
        );

        let actual = wkt.to_geo().unwrap();
        let expected: geo_types::Geometry<f64> = geo_types::Geometry::GeometryCollection(geo_types::GeometryCollection(vec![
            point!(x: 40.0, y: 10.0).into(),
            line_string![(x: 10.0, y: 10.0), (x: 20.0, y: 20.0), (x: 10.0, y: 40.0)].into(),
            polygon![(x: 40.0f64, y: 40.0), (x: 20.0, y: 45.0), (x: 45.0, y: 30.0), (x: 40.0, y: 40.0)].into(),
        ]));
        assert_eq!(expected, actual);
    }

    #[test]
    fn geometry_collection_roundtrip() {
        let str = "GEOMETRYCOLLECTION(POINT(40 10),LINESTRING(10 10,20 20,10 40),POLYGON((40 40,20 45,45 30,40 40)))";
        let wkt = WktStr(str);

        use crate::wkt::conversion::ToWkt;
        let roundtripped = wkt.to_wkt().unwrap();

        assert_eq!(str, &roundtripped);
    }

    mod empties {
        use super::*;

        #[test]
        fn empty_point() {
            let wkt = WktStr("POINT EMPTY");
            let actual = wkt.to_geo().unwrap_err();
            assert!(matches!(actual, GeozeroError::Geometry(_)));
        }

        #[test]
        fn empty_point_roundtrip() {
            let wkt = WktStr("POINT EMPTY");
            let actual = wkt.to_wkt().unwrap();
            assert_eq!("POINT EMPTY", &actual);
        }

        #[test]
        fn geometry_collection_with_empty_point() {
            let str = "GEOMETRYCOLLECTION(POINT(40 10),LINESTRING(10 10,20 20,10 40),POINT EMPTY)";
            let wkt = WktStr(str);

            use crate::wkt::conversion::ToWkt;
            let roundtripped = wkt.to_wkt().unwrap();

            assert_eq!(str, &roundtripped);
        }

        #[test]
        fn empty_line_string() {
            let wkt = WktStr("LINESTRING EMPTY");
            let actual = wkt.to_geo().unwrap();
            let expected: geo_types::Geometry<f64> = line_string![].into();
            assert_eq!(expected, actual);
        }

        #[test]
        fn empty_multi_line_string() {
            let wkt = WktStr("MULTILINESTRING EMPTY");
            let actual = wkt.to_geo().unwrap();
            let expected: geo_types::Geometry<f64> = geo_types::MultiLineString(vec![]).into();
            assert_eq!(expected, actual);
        }

        #[test]
        fn empty_polygon() {
            let wkt = WktStr("POLYGON EMPTY");
            let actual = wkt.to_geo().unwrap();
            let expected: geo_types::Geometry<f64> = polygon![].into();
            assert_eq!(expected, actual);
        }

        #[test]
        fn empty_multi_polygon() {
            let wkt = WktStr("MULTIPOLYGON EMPTY");
            let actual = wkt.to_geo().unwrap();
            let expected: geo_types::Geometry<f64> = geo_types::MultiPolygon(vec![]).into();
            assert_eq!(expected, actual);
        }

        #[test]
        fn empty_geometry_collection() {
            let wkt = WktStr("GEOMETRYCOLLECTION EMPTY");
            let actual = wkt.to_geo().unwrap();

            let expected =
                geo_types::Geometry::GeometryCollection(geo_types::GeometryCollection(vec![]));
            assert_eq!(expected, actual);
        }
    }
}
