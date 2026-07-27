//! Tests for the postgis_diesel geometry types, exercised through the public API.

use postgis_diesel::types::{
    AnyPoint, GeometryCollection, GeometryContainer, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, PointM, PointT, PointZ, PointZM, Polygon,
};

use geozero::wkb::{self, FromWkb};
use geozero::{CoordDimensions, GeozeroGeometry, ToWkb, ToWkt};

const SRID: Option<u32> = Some(4326);

fn points(coords: &[(f64, f64)]) -> Vec<Point> {
    coords
        .iter()
        .map(|&(x, y)| Point::new(x, y, SRID))
        .collect()
}

fn line_string(coords: &[(f64, f64)]) -> LineString<Point> {
    LineString {
        points: points(coords),
        srid: SRID,
    }
}

fn polygon(rings: &[&[(f64, f64)]]) -> Polygon<Point> {
    Polygon {
        rings: rings.iter().map(|ring| points(ring)).collect(),
        srid: SRID,
    }
}

const RING: &[(f64, f64)] = &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.0)];

/// `CoordDimensions` is neither `Debug` nor `PartialEq`, so compare the flags we care about.
fn zm(dims: CoordDimensions) -> (bool, bool) {
    (dims.z, dims.m)
}

/// Encodes via [`GeozeroGeometry`] and decodes via [`FromWkb`], exercising both directions.
fn roundtrip<G, P>(geom: &G) -> GeometryContainer<P>
where
    G: GeozeroGeometry,
    P: PointT,
    GeometryContainer<P>: FromWkb,
{
    let ewkb = geom.to_ewkb(geom.dims(), geom.srid()).unwrap();
    GeometryContainer::<P>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb).unwrap()
}

#[test]
fn geometries_to_wkt() {
    assert_eq!(Point::new(1.0, 2.0, SRID).to_wkt().unwrap(), "POINT(1 2)");
    assert_eq!(
        line_string(&[(0.0, 0.0), (1.0, 1.0)]).to_wkt().unwrap(),
        "LINESTRING(0 0,1 1)"
    );
    assert_eq!(
        polygon(&[RING]).to_wkt().unwrap(),
        "POLYGON((0 0,1 0,1 1,0 0))"
    );

    let multi_point = MultiPoint {
        points: points(&[(1.0, 2.0), (3.0, 4.0)]),
        srid: SRID,
    };
    assert_eq!(multi_point.to_wkt().unwrap(), "MULTIPOINT(1 2,3 4)");

    let multi_line_string = MultiLineString {
        lines: vec![
            line_string(&[(0.0, 0.0), (1.0, 1.0)]),
            line_string(&[(2.0, 2.0), (3.0, 3.0)]),
        ],
        srid: SRID,
    };
    assert_eq!(
        multi_line_string.to_wkt().unwrap(),
        "MULTILINESTRING((0 0,1 1),(2 2,3 3))"
    );

    let multi_polygon = MultiPolygon {
        polygons: vec![polygon(&[RING])],
        srid: SRID,
    };
    assert_eq!(
        multi_polygon.to_wkt().unwrap(),
        "MULTIPOLYGON(((0 0,1 0,1 1,0 0)))"
    );

    let collection = GeometryCollection {
        geometries: vec![
            GeometryContainer::Point(Point::new(1.0, 2.0, SRID)),
            GeometryContainer::LineString(line_string(&[(0.0, 0.0), (1.0, 1.0)])),
        ],
        srid: SRID,
    };
    assert_eq!(
        collection.to_wkt().unwrap(),
        "GEOMETRYCOLLECTION(POINT(1 2),LINESTRING(0 0,1 1))"
    );
}

#[test]
fn polygon_rings_to_wkt() {
    let poly = polygon(&[
        &[
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ],
        &[(1.0, 1.0), (2.0, 1.0), (2.0, 2.0), (1.0, 2.0), (1.0, 1.0)],
    ]);
    assert_eq!(
        poly.to_wkt().unwrap(),
        "POLYGON((0 0,10 0,10 10,0 10,0 0),(1 1,2 1,2 2,1 2,1 1))"
    );
}

#[test]
fn coordinate_dimensions_to_wkt() {
    let xym = CoordDimensions {
        z: false,
        m: true,
        t: false,
        tm: false,
    };

    assert_eq!(Point::new(1.0, 2.0, SRID).to_wkt().unwrap(), "POINT(1 2)");
    assert_eq!(
        PointZ::new(1.0, 2.0, 3.0, SRID)
            .to_wkt_ndim(CoordDimensions::xyz())
            .unwrap(),
        "POINT(1 2 3)"
    );
    // An M value is emitted as the third coordinate when only the M dimension is enabled
    assert_eq!(
        PointM::new(1.0, 2.0, 3.0, SRID).to_wkt_ndim(xym).unwrap(),
        "POINT(1 2 3)"
    );
    assert_eq!(
        PointZM::new(1.0, 2.0, 3.0, 4.0, SRID)
            .to_wkt_ndim(CoordDimensions::xyzm())
            .unwrap(),
        "POINT(1 2 3 4)"
    );
    assert_eq!(
        AnyPoint::PointZ(PointZ::new(1.0, 2.0, 3.0, SRID))
            .to_wkt_ndim(CoordDimensions::xyz())
            .unwrap(),
        "POINT(1 2 3)"
    );

    let line = LineString {
        points: vec![
            PointZ::new(0.0, 0.0, 10.0, SRID),
            PointZ::new(1.0, 1.0, 20.0, SRID),
        ],
        srid: SRID,
    };
    assert_eq!(
        line.to_wkt_ndim(CoordDimensions::xyz()).unwrap(),
        "LINESTRING(0 0 10,1 1 20)"
    );
}

#[test]
fn dims_and_srid_are_reported() {
    let point = Point::new(1.0, 2.0, SRID);
    assert_eq!(GeozeroGeometry::srid(&point), Some(4326));
    assert_eq!(zm(GeozeroGeometry::dims(&point)), (false, false));

    let point_z = PointZ::new(1.0, 2.0, 3.0, Some(7415));
    assert_eq!(GeozeroGeometry::srid(&point_z), Some(7415));
    assert_eq!(zm(GeozeroGeometry::dims(&point_z)), (true, false));

    let measured = PointM::new(1.0, 2.0, 3.0, SRID);
    assert_eq!(zm(GeozeroGeometry::dims(&measured)), (false, true));

    let measured_3d = PointZM::new(1.0, 2.0, 3.0, 4.0, SRID);
    assert_eq!(zm(GeozeroGeometry::dims(&measured_3d)), (true, true));

    // Containers take their dimensions from their first point
    let line = line_string(&[(0.0, 0.0), (1.0, 1.0)]);
    assert_eq!(GeozeroGeometry::srid(&line), Some(4326));
    assert_eq!(zm(GeozeroGeometry::dims(&line)), (false, false));

    // ...and report XY when they are empty
    let empty = LineString::<PointZ> {
        points: Vec::new(),
        srid: SRID,
    };
    assert_eq!(zm(GeozeroGeometry::dims(&empty)), (false, false));
}

#[test]
fn geometry_container_delegates_to_inner_geometry() {
    let container = GeometryContainer::Point(PointZ::new(1.0, 2.0, 3.0, SRID));
    assert_eq!(
        container.to_wkt_ndim(CoordDimensions::xyz()).unwrap(),
        "POINT(1 2 3)"
    );
    assert_eq!(GeozeroGeometry::srid(&container), Some(4326));
    assert_eq!(zm(GeozeroGeometry::dims(&container)), (true, false));

    let container = GeometryContainer::Polygon(polygon(&[RING]));
    assert_eq!(container.to_wkt().unwrap(), "POLYGON((0 0,1 0,1 1,0 0))");
}

#[test]
fn geometries_from_ewkb() {
    let point = Point::new(1.0, 2.0, SRID);
    match roundtrip::<_, Point>(&point) {
        GeometryContainer::Point(p) => {
            assert_eq!((p.x, p.y, p.srid), (1.0, 2.0, SRID));
        }
        other => panic!("Expected Point, got {other:?}"),
    }

    let line = line_string(&[(0.0, 0.0), (1.0, 1.0), (2.0, 3.0)]);
    match roundtrip::<_, Point>(&line) {
        GeometryContainer::LineString(result) => {
            assert_eq!(result.points.len(), 3);
            assert_eq!((result.points[2].x, result.points[2].y), (2.0, 3.0));
            assert_eq!(result.srid, SRID);
        }
        other => panic!("Expected LineString, got {other:?}"),
    }

    let poly = polygon(&[RING]);
    match roundtrip::<_, Point>(&poly) {
        GeometryContainer::Polygon(result) => {
            assert_eq!(result.rings.len(), 1);
            assert_eq!(result.rings[0].len(), 4);
            assert_eq!(result.srid, SRID);
        }
        other => panic!("Expected Polygon, got {other:?}"),
    }

    let multi_point = MultiPoint {
        points: points(&[(1.0, 2.0), (3.0, 4.0)]),
        srid: SRID,
    };
    match roundtrip::<_, Point>(&multi_point) {
        GeometryContainer::MultiPoint(result) => {
            assert_eq!(result.points.len(), 2);
            assert_eq!((result.points[0].x, result.points[1].x), (1.0, 3.0));
            assert_eq!(result.srid, SRID);
        }
        other => panic!("Expected MultiPoint, got {other:?}"),
    }

    let multi_line_string = MultiLineString {
        lines: vec![
            line_string(&[(0.0, 0.0), (1.0, 1.0)]),
            line_string(&[(2.0, 2.0), (3.0, 3.0)]),
        ],
        srid: SRID,
    };
    match roundtrip::<_, Point>(&multi_line_string) {
        GeometryContainer::MultiLineString(result) => {
            assert_eq!(result.lines.len(), 2);
            assert_eq!(result.lines[1].points[0].x, 2.0);
            assert_eq!(result.srid, SRID);
        }
        other => panic!("Expected MultiLineString, got {other:?}"),
    }

    let multi_polygon = MultiPolygon {
        polygons: vec![polygon(&[RING])],
        srid: SRID,
    };
    match roundtrip::<_, Point>(&multi_polygon) {
        GeometryContainer::MultiPolygon(result) => {
            assert_eq!(result.polygons.len(), 1);
            assert_eq!(result.polygons[0].rings[0].len(), 4);
            assert_eq!(result.srid, SRID);
        }
        other => panic!("Expected MultiPolygon, got {other:?}"),
    }
}

#[test]
fn geometry_collection_from_ewkb() {
    let collection = GeometryContainer::GeometryCollection(GeometryCollection {
        geometries: vec![
            GeometryContainer::Point(Point::new(1.0, 2.0, SRID)),
            GeometryContainer::LineString(line_string(&[(0.0, 0.0), (1.0, 1.0)])),
        ],
        srid: SRID,
    });

    match roundtrip::<_, Point>(&collection) {
        GeometryContainer::GeometryCollection(result) => {
            assert_eq!(result.srid, SRID);
            assert!(matches!(
                result.geometries[..],
                [
                    GeometryContainer::Point(_),
                    GeometryContainer::LineString(_)
                ]
            ));
        }
        other => panic!("Expected GeometryCollection, got {other:?}"),
    }
}

#[test]
fn coordinate_dimensions_from_ewkb() {
    match roundtrip::<_, PointZ>(&PointZ::new(1.0, 2.0, 3.0, SRID)) {
        GeometryContainer::Point(p) => {
            assert_eq!((p.x, p.y, p.z, p.srid), (1.0, 2.0, 3.0, SRID))
        }
        other => panic!("Expected PointZ, got {other:?}"),
    }

    match roundtrip::<_, PointM>(&PointM::new(1.0, 2.0, 3.0, SRID)) {
        GeometryContainer::Point(p) => {
            assert_eq!((p.x, p.y, p.m, p.srid), (1.0, 2.0, 3.0, SRID))
        }
        other => panic!("Expected PointM, got {other:?}"),
    }

    match roundtrip::<_, PointZM>(&PointZM::new(1.0, 2.0, 3.0, 4.0, SRID)) {
        GeometryContainer::Point(p) => {
            assert_eq!((p.x, p.y, p.z, p.m, p.srid), (1.0, 2.0, 3.0, 4.0, SRID));
        }
        other => panic!("Expected PointZM, got {other:?}"),
    }

    let any = AnyPoint::PointZ(PointZ::new(1.0, 2.0, 3.0, SRID));
    match roundtrip::<_, AnyPoint>(&any) {
        GeometryContainer::Point(AnyPoint::PointZ(p)) => {
            assert_eq!((p.x, p.y, p.z, p.srid), (1.0, 2.0, 3.0, SRID));
        }
        other => panic!("Expected AnyPoint::PointZ, got {other:?}"),
    }
}

#[cfg(feature = "with-geojson")]
mod geojson {
    use serde_json::Value;

    use super::*;
    use geozero::ToJson;
    use geozero::geojson::GeoJsonString;

    /// The impls have to drive any [`geozero::GeomProcessor`], not just the WKT and WKB writers.
    #[test]
    fn geometries_to_geojson() {
        let json: Value = serde_json::from_str(&polygon(&[RING]).to_json().unwrap()).unwrap();
        assert_eq!(json["type"], "Polygon");
        assert_eq!(json["coordinates"][0][2][0], 1.0);

        let point = PointZ::new(1.0, 2.0, 3.0, SRID);
        let json: Value = serde_json::from_str(&point.to_json().unwrap()).unwrap();
        assert_eq!(json["type"], "Point");
        assert_eq!(json["coordinates"][2], 3.0);
    }

    /// [`FromWkb`] has to accept EWKB produced from any source, not just from postgis_diesel types.
    #[test]
    fn geojson_to_postgis_diesel() {
        let geojson = GeoJsonString(r#"{"type":"Point","coordinates":[1.0,2.0,3.0]}"#.to_string());
        let ewkb = geojson.to_ewkb(CoordDimensions::xyz(), None).unwrap();
        let container =
            GeometryContainer::<AnyPoint>::from_wkb(&mut ewkb.as_slice(), wkb::WkbDialect::Ewkb)
                .unwrap();
        match container {
            GeometryContainer::Point(AnyPoint::PointZ(p)) => {
                assert_eq!((p.x, p.y, p.z), (1.0, 2.0, 3.0));
            }
            other => panic!("Expected AnyPoint::PointZ, got {other:?}"),
        }

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
}
