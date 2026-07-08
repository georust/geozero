use geo::algorithm::{centroid::Centroid, coords_iter::CoordsIter};
use geo::simplify_vw::SimplifyVwPreserve;
use geo::{Geometry, Point};
use geozero::ToGeo;
use geozero::geojson::GeoJson;

// A tiny malformed (E)WKB blob declaring a multi-billion element count must fail
// cleanly instead of reserving tens of gigabytes and aborting (OOM DoS).
#[cfg(feature = "with-wkb")]
#[test]
fn ewkb_huge_count_does_not_oom() {
    use geozero::wkb::Ewkb;

    // EWKB type ids (big-endian) with the high SRID flag set.
    const SRID_FLAG: u32 = 0x2000_0000;
    const LINESTRING: u32 = 2;
    const GEOMETRYCOLLECTION: u32 = 7;

    // Big-endian LineString declaring ~2.95 billion points but carrying no
    // coordinates. Pre-fix this reserved tens of GiB before the read loop.
    let mut linestring = vec![0x00]; // big-endian
    linestring.extend((SRID_FLAG | LINESTRING).to_be_bytes());
    linestring.extend(1i32.to_be_bytes()); // SRID
    linestring.extend(0xb024_a594u32.to_be_bytes()); // lied-about point count
    assert!(Ewkb(linestring.as_slice()).to_geo().is_err());

    // The original fuzz finding: that same LineString wrapped in a
    // GeometryCollection (so the over-large count is reached via a nested header).
    let mut collection = vec![0x00]; // big-endian
    collection.extend((SRID_FLAG | GEOMETRYCOLLECTION).to_be_bytes());
    collection.extend(27i32.to_be_bytes()); // SRID
    collection.extend(1u32.to_be_bytes()); // one member geometry
    collection.extend(linestring);
    assert!(Ewkb(collection.as_slice()).to_geo().is_err());
}

#[test]
fn centroid() {
    let geojson = GeoJson(
        r#"{"type": "Polygon", "coordinates": [[[0, 0], [10, 0], [10, 6], [0, 6], [0, 0]]]}"#,
    );
    if let Ok(Geometry::Polygon(poly)) = geojson.to_geo() {
        assert_eq!(poly.centroid().unwrap(), Point::new(5.0, 3.0));
    }
}

#[test]
fn simplify() {
    let geojson = GeoJson(
        r#"{"type": "LineString", "coordinates": [[1875038.447610231,-3269648.6879248763],[1874359.641504197,-3270196.812984864],[1874141.0428635243,-3270953.7840121365],[1874440.1778162003,-3271619.4315206874],[1876396.0598222911,-3274138.747656357],[1876442.0805243007,-3275052.60551469],[1874739.312657555,-3275457.333765534]]}"#,
    );
    if let Ok(Geometry::LineString(line)) = geojson.to_geo() {
        assert_eq!(line.coords_count(), 7);
        let simplified = line.simplify_vw_preserve(800000.0);
        assert_eq!(simplified.coords_count(), 4);
    }
}
