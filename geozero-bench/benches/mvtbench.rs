use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geozero::error::Result;
use geozero::mvt::{Message, Tile};
use geozero::{
    ColumnValue, CoordDimensions, FeatureProcessor, GeomProcessor, GeozeroDatasource,
    PropertyProcessor,
};

struct Proc {
    dimensions: CoordDimensions,
    multi_dim: bool,
}

impl Proc {
    fn new(dimensions: CoordDimensions, multi_dim: bool) -> Self {
        Self {
            dimensions,
            multi_dim,
        }
    }
}

impl GeomProcessor for Proc {
    fn dimensions(&self) -> CoordDimensions {
        self.dimensions
    }

    fn multi_dim(&self) -> bool {
        self.multi_dim
    }

    fn srid(&mut self, srid: Option<i32>) -> Result<()> {
        black_box(srid);
        Ok(())
    }

    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        black_box(x);
        black_box(y);
        black_box(idx);
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
    ) -> Result<()> {
        black_box(x);
        black_box(y);
        black_box(z);
        black_box(m);
        black_box(t);
        black_box(tm);
        black_box(idx);
        Ok(())
    }

    fn empty_point(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn point_begin(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn point_end(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        black_box(tagged);
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        black_box(tagged);
        black_box(idx);
        Ok(())
    }

    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        black_box(tagged);
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        black_box(tagged);
        black_box(idx);
        Ok(())
    }

    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn circularstring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn circularstring_end(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn compoundcurve_end(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn curvepolygon_end(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn multicurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn multicurve_end(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn multisurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn multisurface_end(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn triangle_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        black_box(tagged);
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn triangle_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        black_box(tagged);
        black_box(idx);
        Ok(())
    }

    fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn polyhedralsurface_end(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        black_box(size);
        black_box(idx);
        Ok(())
    }

    fn tin_end(&mut self, idx: usize) -> Result<()> {
        black_box(idx);
        Ok(())
    }
}

impl PropertyProcessor for Proc {
    fn property(&mut self, idx: usize, name: &str, value: &ColumnValue) -> Result<bool> {
        black_box(idx);
        black_box(name);
        black_box(value);
        Ok(true)
    }
}

impl FeatureProcessor for Proc {
    fn dataset_begin(&mut self, name: Option<&str>) -> Result<()> {
        black_box(name);
        Ok(())
    }

    fn dataset_end(&mut self) -> Result<()> {
        black_box(true);
        Ok(())
    }

    fn feature_begin(&mut self, idx: u64) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn feature_end(&mut self, idx: u64) -> Result<()> {
        black_box(idx);
        Ok(())
    }

    fn properties_begin(&mut self) -> Result<()> {
        black_box(true);
        Ok(())
    }

    fn properties_end(&mut self) -> Result<()> {
        black_box(true);
        Ok(())
    }

    fn geometry_begin(&mut self) -> Result<()> {
        black_box(true);
        Ok(())
    }

    fn geometry_end(&mut self) -> Result<()> {
        black_box(true);
        Ok(())
    }
}

fn run_parse(mut proc: Proc) {
    let data = &include_bytes!("../../geozero/tests/data/tile.mvt")[..];
    let tile = Tile::decode(black_box(data)).unwrap();
    for mut layer in tile.layers {
        layer.process(&mut proc).unwrap();
    }
}

fn mvt_benchmark(c: &mut Criterion) {
    c.bench_function("mvt decoding xy", |b| {
        b.iter(|| run_parse(Proc::new(CoordDimensions::xy(), false)))
    });

    c.bench_function("mvt decoding xy multi_dim", |b| {
        b.iter(|| run_parse(Proc::new(CoordDimensions::xy(), true)))
    });
}

criterion_group!(benches, mvt_benchmark);
criterion_main!(benches);
