use geozero::error::Result;
use geozero::geojson::GeoJsonReader;
use geozero::GeozeroDatasource;
use kdbush::*;
use std::fs::File;

struct PointIndex {
    pos: usize,
    index: KDBush,
}

impl geozero::GeomProcessor for PointIndex {
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
        self.index.add_point(self.pos, x, y);
        self.pos += 1;
        Ok(())
    }
}

#[test]
fn create() -> Result<()> {
    let f = File::open("tests/data/places.json")?;
    let mut reader = GeoJsonReader(f);
    let mut points = PointIndex {
        pos: 0,
        index: KDBush::new(1249, DEFAULT_NODE_SIZE),
    };
    reader.process_geom(&mut points)?;
    points.index.build_index();
    let mut cnt = 0;
    points.index.within(8.53, 47.37, 5.0, |_id| cnt += 1);
    assert_eq!(cnt, 22);
    Ok(())
}
