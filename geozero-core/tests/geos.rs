#[cfg(feature = "geos-lib")]
mod geos_examples {

    use geos::Geom;
    use geozero_core::geojson::GeoJson;
    use geozero_core::ToGeos;

    #[test]
    fn prepared_geom() {
        let geojson = GeoJson(
            r#"{"type": "Polygon", "coordinates": [[[0, 0], [10, 0], [10, 6], [0, 6], [0, 0]]]}"#
                .to_string(),
        );
        let geom = geojson.to_geos().expect("GEOS conversion failed");
        let prepared_geom = geom.to_prepared_geom().expect("to_prepared_geom failed");
        let geom2 = geos::Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
        assert_eq!(prepared_geom.contains(&geom2), Ok(true));
    }
}
