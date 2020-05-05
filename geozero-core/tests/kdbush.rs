use geozero::error::Result;
use geozero_core::geojson::read_geojson_geom;
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
#[ignore]
fn create() -> Result<()> {
    let f = File::open("tests/data/cities.geojson")?;
    let mut points = PointIndex {
        pos: 0,
        index: KDBush::new(708024, DEFAULT_NODE_SIZE),
    };
    read_geojson_geom(f, &mut points)?;
    points.index.build_index();
    let mut cnt = 0;
    points.index.within(8.53, 47.37, 0.1, |_id| cnt += 1);
    assert_eq!(cnt, 190);
    Ok(())
}
