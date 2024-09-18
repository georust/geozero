use geozero::error::Result as GeozeroResult;
use geozero::wkt::Wkt;
use geozero::{CoordDimensions, GeomProcessor, GeozeroGeometry};

struct VertexCounter(u64);

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl GeomProcessor for VertexCounter {
    fn xy(&mut self, _x: f64, _y: f64, _idx: usize) -> GeozeroResult<()> {
        self.0 += 1;
        Ok(())
    }
}

#[test]
fn vertex_counter() -> Result<()> {
    let wkt = Wkt("MULTIPOLYGON (((40 40, 20 45, 45 30, 40 40)),
                         ((35 10, 45 45, 15 40, 10 20, 35 10),
                         (20 30, 35 35, 30 20, 20 30)))");

    let mut vertex_counter = VertexCounter(0);
    wkt.process_geom(&mut vertex_counter)?;
    assert_eq!(vertex_counter.0, 13);

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
    ) -> GeozeroResult<()> {
        if let Some(z) = z {
            if z > self.0 {
                self.0 = z
            }
        }
        Ok(())
    }
}

#[test]
fn max_height_finder() -> Result<()> {
    let points = Wkt("MULTIPOINTZ (10 40 200.0, 40 30 150.0, 20 20 457.1, 30 10 0.0)");
    let mut max_finder = MaxHeightFinder(0.0);
    points.process_geom(&mut max_finder)?;
    assert_eq!(max_finder.0, 457.1);

    Ok(())
}

/*
// Broken: this example does work with Flatgeobuf, but not GeoJson
// Disabled, since we don't want to have a circular dependency between flatgeobuf and geozero

struct FeatureFinder;

impl PropertyProcessor for FeatureFinder {
    fn property(&mut self, i: usize, _name: &str, v: &ColumnValue) -> GeozeroResult<bool> {
        Ok(i == 0 && v == &ColumnValue::String("DNK"))
    }
}

#[test]
fn feature_finder() -> Result<()> {
    let mut filein = BufReader::new(File::open("tests/data/countries.geojson")?);
    let mut json = GeoJsonLineReader::new(&mut filein);

    let mut finder = FeatureFinder {};
    // process_properties is not public. We should have feature iterators for all `GeozeroDatasource`s!
    // json.process_properties(&mut finder);
    //
    // Using FgbReader:
    // while let Some(feature) = fgb.next()? {
    //     let found = feature.process_properties(&mut finder);
    //     if found.is_err() || found.unwrap() {
    //         break;
    //     }
    // }
    // let feature = fgb.cur_feature();
    // let props = feature.properties()?;
    // assert_eq!(props["id"], "DNK".to_string());
    // assert_eq!(props["name"], "Denmark".to_string());

    Ok(())
}
*/
