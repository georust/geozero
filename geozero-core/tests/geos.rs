#[cfg(feature = "geos-lib")]
mod geos_examples {

    use geos::Geom;
    use geozero_core::geojson::read_geojson;
    use geozero_core::geos::Geos;

    #[test]
    fn prepared_geom() {
        let geojson =
            r#"{"type": "Polygon", "coordinates": [[[0, 0], [10, 0], [10, 6], [0, 6], [0, 0]]]}"#;
        let mut geos = Geos::new();
        assert!(read_geojson(geojson.as_bytes(), &mut geos).is_ok());
        let prepared_geom = geos
            .geometry()
            .to_prepared_geom()
            .expect("to_prepared_geom failed");
        let geom2 = geos::Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
        assert_eq!(prepared_geom.contains(&geom2), Ok(true));
    }
}
