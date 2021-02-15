use geo::algorithm::{centroid::Centroid, coords_iter::CoordsIter};
use geo::simplifyvw::SimplifyVWPreserve;
use geo::{Geometry, Point};
use geozero_core::geo_types::GeoWriter;
use geozero_core::geojson::read_geojson_geom;

#[test]
fn centroid() {
    let geojson = r#"{"type": "LineString", "coordinates": [[1875038.447610231,-3269648.6879248763],[1874359.641504197,-3270196.812984864],[1874141.0428635243,-3270953.7840121365],[1874440.1778162003,-3271619.4315206874],[1876396.0598222911,-3274138.747656357],[1876442.0805243007,-3275052.60551469],[1874739.312657555,-3275457.333765534]]}"#;
    let mut geo = GeoWriter::new();
    assert!(read_geojson_geom(geojson.as_bytes(), &mut geo).is_ok());
    if let Geometry::LineString(line) = geo.geometry() {
        assert_eq!(
            line.centroid().unwrap(),
            Point::new(1875278.3503806225, -3272899.9912620876)
        );
    }
}

#[test]
fn simplify() {
    let geojson = r#"{"type": "LineString", "coordinates": [[1875038.447610231,-3269648.6879248763],[1874359.641504197,-3270196.812984864],[1874141.0428635243,-3270953.7840121365],[1874440.1778162003,-3271619.4315206874],[1876396.0598222911,-3274138.747656357],[1876442.0805243007,-3275052.60551469],[1874739.312657555,-3275457.333765534]]}"#;
    let mut geo = GeoWriter::new();
    assert!(read_geojson_geom(geojson.as_bytes(), &mut geo).is_ok());
    if let Geometry::LineString(line) = geo.geometry() {
        assert_eq!(line.coords_count(), 7);
        let simplified = line.simplifyvw_preserve(&800000.0);
        assert_eq!(simplified.coords_count(), 4);
    }
}
