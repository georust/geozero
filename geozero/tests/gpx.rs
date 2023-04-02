use geozero::gpx::{Gpx, GpxReader};

use std::io;

mod test_writer;

use test_writer::{Cmd, TestWriter};

#[test]
fn test_empty_str() {
    let gpx_str = "";
    let mut cursor = io::Cursor::new(gpx_str);
    let mut writer = TestWriter::default();

    assert!(matches!(
        geozero::gpx::read_gpx(&mut cursor, &mut writer),
        Err(geozero::error::GeozeroError::Geometry(_)),
    ));
}

#[test]
fn test_extensive() {
    let gpx_str = include_str!("data/extensive.gpx");
    let mut cursor = io::Cursor::new(gpx_str);
    let mut writer = TestWriter::default();

    geozero::gpx::read_gpx(&mut cursor, &mut writer).unwrap();

    #[rustfmt::skip]
    assert_eq!(
        writer.0,
        vec![
            Cmd::GeometryCollectionBegin { idx: 0, size: 4 },
                Cmd::PointBegin { idx: 0 },
                    Cmd::Xy { idx: 0, x: -1.5153741828293, y: 47.253146555709 },
                Cmd::PointEnd { idx: 0 },
                Cmd::PointBegin { idx: 1 },
                    Cmd::Xy { idx: 0, x: -1.5482325613225, y: 47.235331031612 },
                Cmd::PointEnd { idx: 1 },
                Cmd::MultiLineStringBegin { idx: 2 },
                    Cmd::LineStringBegin { idx: 0 },
                        Cmd::Xy { idx: 0, x: -1.5521714646550901, y: 47.2278526991611 },
                        Cmd::Xy { idx: 1, x: -1.5504753767742476, y: 47.229236980562256 },
                    Cmd::LineStringEnd { idx: 0 },
                    Cmd::LineStringBegin { idx: 1 },
                        Cmd::Xy { idx: 0, x: -1.5493804339650867, y: 47.2301112449252 },
                        Cmd::Xy { idx: 1, x: -1.5485645942249218, y: 47.230562942529104 },
                    Cmd::LineStringEnd { idx: 1 },
                Cmd::MultiLineStringEnd { idx: 2 },
                Cmd::MultiLineStringBegin { idx: 3 },
                    Cmd::LineStringBegin { idx: 0 },
                        Cmd::Xy { idx: 0, x: -1.5521714646550901, y: 47.2278526991611 },
                        Cmd::Xy { idx: 1, x: -1.5504753767742476, y: 47.229236980562256 },
                        Cmd::Xy { idx: 2, x: -1.5493804339650867, y: 47.2301112449252 },
                    Cmd::LineStringEnd { idx: 0 },
                Cmd::MultiLineStringEnd { idx: 3 },
            Cmd::GeometryCollectionEnd { idx: 0 },
        ]
    );
}

#[test]
fn test_wikipedia_example() {
    let gpx_str = include_str!("data/wikipedia_example.gpx");
    let mut cursor = io::Cursor::new(gpx_str);
    let mut writer = TestWriter::default();

    geozero::gpx::read_gpx(&mut cursor, &mut writer).unwrap();

    #[rustfmt::skip]
    assert_eq!(
        writer.0,
        vec![
            Cmd::GeometryCollectionBegin { idx: 0, size: 1 },
                Cmd::MultiLineStringBegin { idx: 0 },
                    Cmd::LineStringBegin { idx: 0 },
                        Cmd::Xy { idx: 0, x: -122.326897, y: 47.644548, },
                        Cmd::Xy { idx: 1, x: -122.326897, y: 47.644548, },
                        Cmd::Xy { idx: 2, x: -122.326897, y: 47.644548, },
                    Cmd::LineStringEnd { idx: 0 },
                Cmd::MultiLineStringEnd { idx: 0 },
            Cmd::GeometryCollectionEnd { idx: 0 },
        ]
    );
}

mod wikipedia_example_conversions {
    use super::*;

    #[test]
    fn to_geojson() {
        let gpx_str = include_str!("data/wikipedia_example.gpx");
        let mut cursor = io::Cursor::new(gpx_str);
        let mut reader = GpxReader(&mut cursor);

        use geozero::ProcessToJson;
        let geojson = reader.to_json().unwrap();
        assert_eq!(
            r#"{"type": "GeometryCollection", "geometries": [{"type": "MultiLineString", "coordinates": [[[-122.326897,47.644548],[-122.326897,47.644548],[-122.326897,47.644548]]]}]}"#,
            geojson
        );
    }

    #[test]
    fn to_svg() {
        let gpx_str = include_str!("data/wikipedia_example.gpx");
        let mut cursor = io::Cursor::new(gpx_str);
        let mut reader = GpxReader(&mut cursor);

        use geozero::ProcessToSvg;
        let geojson = reader.to_svg().unwrap();
        assert_eq!(
            r#"<path d="M -122.326897 47.644548 -122.326897 47.644548 -122.326897 47.644548 Z "/>"#,
            geojson
        );
    }

    #[test]
    fn to_wkt() {
        let gpx_str = include_str!("data/wikipedia_example.gpx");
        let reader = Gpx(gpx_str);

        use geozero::ToWkt;
        let wkt = reader.to_wkt().unwrap();
        assert_eq!(
            r#"GEOMETRYCOLLECTION(MULTILINESTRING((-122.326897 47.644548,-122.326897 47.644548,-122.326897 47.644548)))"#,
            wkt
        );
    }
}

mod extensive_conversion {
    use super::*;

    #[test]
    fn to_geojson() {
        let gpx_str = include_str!("data/extensive.gpx");
        let mut cursor = io::Cursor::new(gpx_str);
        let mut reader = GpxReader(&mut cursor);

        use geozero::ProcessToJson;
        let geojson = reader.to_json().unwrap();
        assert_eq!(
            r#"{"type": "GeometryCollection", "geometries": [{"type": "Point", "coordinates": [-1.5153741828293,47.253146555709]},{"type": "Point", "coordinates": [-1.5482325613225,47.235331031612]},{"type": "MultiLineString", "coordinates": [[[-1.5521714646550901,47.2278526991611],[-1.5504753767742476,47.229236980562256]],[[-1.5493804339650867,47.2301112449252],[-1.5485645942249218,47.230562942529104]]]},{"type": "MultiLineString", "coordinates": [[[-1.5521714646550901,47.2278526991611],[-1.5504753767742476,47.229236980562256],[-1.5493804339650867,47.2301112449252]]]}]}"#,
            geojson
        );
    }

    #[test]
    fn to_svg() {
        let gpx_str = include_str!("data/extensive.gpx");
        let reader = Gpx(gpx_str);

        use geozero::ToSvg;
        let actual_svg = reader.to_svg().unwrap();
        let expected_svg: &str = r#"<path d="M -1.5153741828293 47.253146555709 Z"/><path d="M -1.5482325613225 47.235331031612 Z"/><path d="M -1.5521714646550901 47.2278526991611 -1.5504753767742476 47.229236980562256 Z M -1.5493804339650867 47.2301112449252 -1.5485645942249218 47.230562942529104 Z "/><path d="M -1.5521714646550901 47.2278526991611 -1.5504753767742476 47.229236980562256 -1.5493804339650867 47.2301112449252 Z "/>"#;
        assert_eq!(expected_svg, actual_svg);
    }

    #[test]
    fn to_wkt() {
        let gpx_str = include_str!("data/extensive.gpx");
        let reader = Gpx(gpx_str);

        use geozero::ToWkt;
        let wkt = reader.to_wkt().unwrap();
        let expected_wkt: &str = "GEOMETRYCOLLECTION(POINT(-1.5153741828293 47.253146555709),POINT(-1.5482325613225 47.235331031612),MULTILINESTRING((-1.5521714646550901 47.2278526991611,-1.5504753767742476 47.229236980562256),(-1.5493804339650867 47.2301112449252,-1.5485645942249218 47.230562942529104)),MULTILINESTRING((-1.5521714646550901 47.2278526991611,-1.5504753767742476 47.229236980562256,-1.5493804339650867 47.2301112449252)))";
        assert_eq!(expected_wkt, wkt);
    }
}
