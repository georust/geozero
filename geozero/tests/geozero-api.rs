use flatgeobuf::{FallibleStreamingIterator as _, FeatureProperties as _, FgbReader, GeometryType};
use geozero::error::Result;
use geozero::{ColumnValue, CoordDimensions, GeomProcessor, PropertyProcessor};
use seek_bufread::BufReader;
use std::fs::File;

struct VertexCounter(u64);

impl GeomProcessor for VertexCounter {
    fn xy(&mut self, _x: f64, _y: f64, _idx: usize) -> Result<()> {
        self.0 += 1;
        Ok(())
    }
}

#[test]
fn vertex_counter() -> Result<()> {
    let mut filein = BufReader::new(File::open("tests/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?.select_bbox(8.8, 47.2, 9.5, 55.3)?;
    let feature = fgb.next()?.unwrap();
    let geometry = feature.geometry().unwrap();

    let mut vertex_counter = VertexCounter(0);
    geometry.process(&mut vertex_counter, GeometryType::MultiPolygon)?;
    assert_eq!(vertex_counter.0, 24);

    Ok(())
}

struct MaxHeightFinder(f64);

impl GeomProcessor for MaxHeightFinder {
    fn dimensions(&self) -> CoordDimensions {
        CoordDimensions {
            z: true,
            m: false,
            t: false,
            tm: false,
        }
    }
    fn coordinate(
        &mut self,
        _x: f64,
        _y: f64,
        z: Option<f64>,
        _m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
        _idx: usize,
    ) -> Result<()> {
        if let Some(z) = z {
            if z > self.0 {
                self.0 = z
            }
        }
        Ok(())
    }
}

#[test]
#[ignore]
fn max_height_finder() -> Result<()> {
    let mut filein = BufReader::new(File::open(
        "tests/data/geoz_lod1_gebaeude_max_3d_extract.fgb",
    )?);
    let mut fgb = FgbReader::open(&mut filein)?.select_all()?;
    let mut max_finder = MaxHeightFinder(0.0);
    while let Some(feature) = fgb.next()? {
        let geometry = feature.geometry().unwrap();
        geometry.process(&mut max_finder, GeometryType::MultiPolygon)?;
    }
    assert_eq!(max_finder.0, 457.1);

    Ok(())
}

struct FeatureFinder;

impl PropertyProcessor for FeatureFinder {
    fn property(&mut self, i: usize, _name: &str, v: &ColumnValue) -> Result<bool> {
        Ok(i == 0 && v == &ColumnValue::String("DNK"))
    }
}

#[test]
fn feature_finder() -> Result<()> {
    let mut filein = BufReader::new(File::open("tests/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?.select_all()?;

    let mut finder = FeatureFinder {};
    while let Some(feature) = fgb.next()? {
        let found = feature.process_properties(&mut finder);
        if found.is_err() || found.unwrap() {
            break;
        }
    }
    let feature = fgb.cur_feature();
    let props = feature.properties()?;
    assert_eq!(props["id"], "DNK".to_string());
    assert_eq!(props["name"], "Denmark".to_string());

    Ok(())
}
