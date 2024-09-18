use geozero::{wkt::Wkt, ToJson};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn wkt_to_geojson_feature() -> Result<()> {
    let wkt = Wkt("MULTIPOLYGON (((40 40, 20 45, 45 30, 40 40)),
                         ((35 10, 45 45, 15 40, 10 20, 35 10),
                         (20 30, 35 35, 30 20, 20 30)))");
    let json = wkt.to_json()?;

    println!("{json}");
    assert_eq!(
        &json,
        r#"{"type": "MultiPolygon", "coordinates": [[[[40,40],[20,45],[45,30],[40,40]]],[[[35,10],[45,45],[15,40],[10,20],[35,10]],[[20,30],[35,35],[30,20],[20,30]]]]}"#
    );
    Ok(())
}
