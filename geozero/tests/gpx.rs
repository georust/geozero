use std::io;

mod test_writer;

use test_writer::{TestWriter, Cmd};

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
            Cmd::MultiPointBegin { idx: 0 },
                Cmd::Xy { idx: 0, x: -1.5153741828293, y: 47.253146555709 },
                Cmd::Xy { idx: 1, x: -1.5482325613225, y: 47.235331031612 },
            Cmd::MultiPointEnd { idx: 0 },
            Cmd::MultiLineStringBegin { idx: 0 },
                Cmd::LineStringBegin { idx: 0 },
                    Cmd::Xy { idx: 0, x: -1.5521714646550901, y: 47.2278526991611 },
                    Cmd::Xy { idx: 1, x: -1.5504753767742476, y: 47.229236980562256 },
                Cmd::LineStringEnd { idx: 0 },
                Cmd::LineStringBegin { idx: 1 },
                    Cmd::Xy { idx: 0, x: -1.5493804339650867, y: 47.2301112449252 },
                    Cmd::Xy { idx: 1, x: -1.5485645942249218, y: 47.230562942529104 },
                Cmd::LineStringEnd { idx: 1 },
            Cmd::MultiLineStringEnd { idx: 0 },
            Cmd::MultiLineStringBegin { idx: 0 },
                Cmd::LineStringBegin { idx: 0 },
                    Cmd::Xy { idx: 0, x: -1.5521714646550901, y: 47.2278526991611 },
                    Cmd::Xy { idx: 1, x: -1.5504753767742476, y: 47.229236980562256 },
                    Cmd::Xy { idx: 2, x: -1.5493804339650867, y: 47.2301112449252 },
                Cmd::LineStringEnd { idx: 0 },
            Cmd::MultiLineStringEnd { idx: 0 },
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
            Cmd::MultiLineStringBegin { idx: 0 },
                Cmd::LineStringBegin { idx: 0 },
                    Cmd::Xy { idx: 0, x: -122.326897, y: 47.644548, },
                    Cmd::Xy { idx: 1, x: -122.326897, y: 47.644548, },
                    Cmd::Xy { idx: 2, x: -122.326897, y: 47.644548, },
                Cmd::LineStringEnd { idx: 0 },
            Cmd::MultiLineStringEnd { idx: 0 },
        ]
    );
}
