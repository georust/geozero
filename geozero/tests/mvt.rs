use geozero::mvt::{Message, Tile};
use geozero::{
    ColumnValue, CoordDimensions, FeatureProcessor, GeomProcessor, GeozeroDatasource,
    PropertyProcessor, ToJson, ToMvt,
};
use serde_json::json;
use std::env;
use std::fmt::Write;
use std::sync::Mutex;

#[test]
fn geo_screen_coords_to_mvt() {
    let geo: geo_types::Geometry<f64> = geo_types::Point::new(25.0, 17.0).into();
    let mvt = geo.to_mvt_unscaled().unwrap();
    assert_eq!(mvt.geometry, [9, 50, 34]);
}

#[test]
fn geo_to_mvt() {
    let geo: geo_types::Geometry<f64> = geo_types::Point::new(960000.0, 6002729.0).into();
    let mvt = geo
        .to_mvt(256, 958826.08, 5987771.04, 978393.96, 6007338.92)
        .unwrap();
    assert_eq!(mvt.geometry, [9, 30, 122]);
    let geojson = mvt.to_json().unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&geojson).unwrap(),
        json!({
            "type": "Point",
            "coordinates": [15,61]
        })
    );
}

type GzResult = geozero::error::Result<()>;

struct Proc {
    buf: String,
    dimensions: Mutex<usize>,
    multi_dim: Mutex<usize>,
}

impl Proc {
    fn new(buf: String) -> Self {
        Self {
            buf,
            dimensions: Mutex::new(0),
            multi_dim: Mutex::new(0),
        }
    }
}

impl GeomProcessor for Proc {
    fn dimensions(&self) -> CoordDimensions {
        let mut dimensions = self.dimensions.lock().unwrap();
        *dimensions += 1;
        CoordDimensions::xy()
    }

    fn multi_dim(&self) -> bool {
        let mut multi_dim = self.multi_dim.lock().unwrap();
        *multi_dim += 1;
        false
    }

    fn srid(&mut self, srid: Option<i32>) -> GzResult {
        writeln!(self.buf, "srid: {srid:?}").unwrap();
        Ok(())
    }

    fn xy(&mut self, x: f64, y: f64, idx: usize) -> GzResult {
        writeln!(self.buf, "      xy: {x}, {y}, {idx}").unwrap();
        Ok(())
    }

    fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        m: Option<f64>,
        t: Option<f64>,
        tm: Option<u64>,
        idx: usize,
    ) -> GzResult {
        writeln!(
            &mut self.buf,
            "      coordinate: {x}, {y}, {z:?}, {m:?}, {t:?}, {tm:?}, {idx}"
        )
        .unwrap();
        Ok(())
    }

    fn empty_point(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "empty_point {idx}").unwrap();
        Ok(())
    }

    fn point_begin(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "    point_begin {idx}").unwrap();
        Ok(())
    }

    fn point_end(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "    point_end {idx}").unwrap();
        Ok(())
    }

    fn multipoint_begin(&mut self, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "    multipoint_begin {size}, {idx}").unwrap();
        Ok(())
    }

    fn multipoint_end(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "    multipoint_end {idx}").unwrap();
        Ok(())
    }

    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "linestring_begin {tagged}, {size}, {idx}").unwrap();
        Ok(())
    }

    fn linestring_end(&mut self, tagged: bool, idx: usize) -> GzResult {
        writeln!(self.buf, "linestring_end {tagged}, {idx}").unwrap();
        Ok(())
    }

    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "multilinestring_begin {size}, {idx}").unwrap();
        Ok(())
    }

    fn multilinestring_end(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "multilinestring_end {idx}").unwrap();
        Ok(())
    }

    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "polygon_begin {tagged}, {size}, {idx}").unwrap();
        Ok(())
    }

    fn polygon_end(&mut self, tagged: bool, idx: usize) -> GzResult {
        writeln!(self.buf, "polygon_end {tagged}, {idx}").unwrap();
        Ok(())
    }

    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "multipolygon_begin {size}, {idx}").unwrap();
        Ok(())
    }

    fn multipolygon_end(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "multipolygon_end {idx}").unwrap();
        Ok(())
    }

    fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "geometrycollection_begin {size}, {idx}").unwrap();
        Ok(())
    }

    fn geometrycollection_end(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "geometrycollection_end {idx}").unwrap();
        Ok(())
    }

    fn circularstring_begin(&mut self, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "circularstring_begin {size}, {idx}").unwrap();
        Ok(())
    }

    fn circularstring_end(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "circularstring_end {idx}").unwrap();
        Ok(())
    }

    fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "compoundcurve_begin {size}, {idx}").unwrap();
        Ok(())
    }

    fn compoundcurve_end(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "compoundcurve_end {idx}").unwrap();
        Ok(())
    }

    fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "curvepolygon_begin {size}, {idx}").unwrap();
        Ok(())
    }

    fn curvepolygon_end(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "curvepolygon_end {idx}").unwrap();
        Ok(())
    }

    fn multicurve_begin(&mut self, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "multicurve_begin {size}, {idx}").unwrap();
        Ok(())
    }

    fn multicurve_end(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "multicurve_end {idx}").unwrap();
        Ok(())
    }

    fn multisurface_begin(&mut self, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "multisurface_begin {size}, {idx}").unwrap();
        Ok(())
    }

    fn multisurface_end(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "multisurface_end {idx}").unwrap();
        Ok(())
    }

    fn triangle_begin(&mut self, tagged: bool, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "triangle_begin {tagged}, {size}, {idx}").unwrap();
        Ok(())
    }

    fn triangle_end(&mut self, tagged: bool, idx: usize) -> GzResult {
        writeln!(self.buf, "triangle_end {tagged}, {idx}").unwrap();
        Ok(())
    }

    fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "polyhedralsurface_begin {size}, {idx}").unwrap();
        Ok(())
    }

    fn polyhedralsurface_end(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "polyhedralsurface_end {idx}").unwrap();
        Ok(())
    }

    fn tin_begin(&mut self, size: usize, idx: usize) -> GzResult {
        writeln!(self.buf, "tin_begin {size}, {idx}").unwrap();
        Ok(())
    }

    fn tin_end(&mut self, idx: usize) -> GzResult {
        writeln!(self.buf, "tin_end {idx}").unwrap();
        Ok(())
    }
}

impl PropertyProcessor for Proc {
    fn property(
        &mut self,
        idx: usize,
        name: &str,
        value: &ColumnValue,
    ) -> geozero::error::Result<bool> {
        writeln!(self.buf, "    property {idx}: {name} = {value:?}").unwrap();
        Ok(true)
    }
}

impl FeatureProcessor for Proc {
    fn dataset_begin(&mut self, name: Option<&str>) -> geozero::error::Result<()> {
        writeln!(self.buf, "dataset_begin {name:?}").unwrap();
        Ok(())
    }

    fn dataset_end(&mut self) -> geozero::error::Result<()> {
        writeln!(self.buf, "dataset_end").unwrap();
        Ok(())
    }

    fn feature_begin(&mut self, idx: u64) -> geozero::error::Result<()> {
        writeln!(self.buf, "feature_begin {idx}").unwrap();
        Ok(())
    }

    fn feature_end(&mut self, idx: u64) -> geozero::error::Result<()> {
        writeln!(self.buf, "feature_end {idx}").unwrap();
        Ok(())
    }

    fn properties_begin(&mut self) -> geozero::error::Result<()> {
        writeln!(self.buf, "  properties_begin").unwrap();
        Ok(())
    }

    fn properties_end(&mut self) -> geozero::error::Result<()> {
        writeln!(self.buf, "  properties_end").unwrap();
        Ok(())
    }

    fn geometry_begin(&mut self) -> geozero::error::Result<()> {
        writeln!(self.buf, "  geometry_begin").unwrap();
        Ok(())
    }

    fn geometry_end(&mut self) -> geozero::error::Result<()> {
        writeln!(self.buf, "  geometry_end").unwrap();
        Ok(())
    }
}

/// Parse data/tile.mvt and print its content, keeping track of all callbacks.
/// The output is compared with the data/tile.mvt.txt, and if it differs, the test saves new file and fails.
#[test]
fn mvt_decode() {
    let data = &include_bytes!("data/tile.mvt")[..];

    let mut buf = String::new();
    let tile = Tile::decode(data).unwrap();
    for mut layer in tile.layers {
        let mut proc = Proc::new(buf);
        writeln!(proc.buf, "---------- start layer {} ----------", layer.name).unwrap();
        layer.process(&mut proc).unwrap();
        let dimensions = *proc.dimensions.lock().unwrap();
        writeln!(proc.buf, "::: dimensions = {dimensions}").unwrap();
        let multi_dim = *proc.multi_dim.lock().unwrap();
        writeln!(proc.buf, "::: multi_dim = {multi_dim}").unwrap();
        writeln!(proc.buf, "---------- end layer {} ----------", layer.name).unwrap();
        buf = proc.buf;
    }

    let test_dir = env::current_dir().unwrap().join("tests").join("data");
    let expected_file = test_dir.join("tile.mvt.txt");
    let new_file = test_dir.join("tile.mvt.new.txt");

    let expected = std::fs::read_to_string(&expected_file).ok();
    if expected.filter(|e| e == &buf).is_none() {
        std::fs::write(&new_file, buf).unwrap_or_else(|e| {
            panic!(
                "{expected_file:?} didn't match mvt output, and failed to write {new_file:?}: {e}"
            )
        });
        panic!("{expected_file:?} didn't match mvt output.  See {new_file:?} file for the new output, and if it is correct, replace {expected_file:?} with it");
    }
}
