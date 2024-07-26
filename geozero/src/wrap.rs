use crate::{
    error::Result, ColumnValue, CoordDimensions, FeatureProcessor, GeomProcessor, PropertyProcessor,
};

/// Wraps another [`FeatureProcessor`], first transforming coordinates.
pub struct WrappedProcessor<T, F: Fn(f64, f64) -> (f64, f64)> {
    /// The underlying FeatureProcessor
    pub inner: T,
    transform_coordinates: F,
}

impl<T, F: Fn(f64, f64) -> (f64, f64)> WrappedProcessor<T, F> {
    /// Wraps an inner [`FeatureProcessor`], calling `transform_coordinates` on [GeomProcessor::xy]
    /// and [GeomProcessor::coordinate] first. The function takes and returns `(x, y)`.
    pub fn new(inner: T, transform_coordinates: F) -> Self {
        Self {
            inner,
            transform_coordinates,
        }
    }
}

// The trait has many default implementations, but every single call must be specified here to
// delegate
impl<T: GeomProcessor, F: Fn(f64, f64) -> (f64, f64)> GeomProcessor for WrappedProcessor<T, F> {
    fn dimensions(&self) -> CoordDimensions {
        self.inner.dimensions()
    }
    fn multi_dim(&self) -> bool {
        self.inner.multi_dim()
    }
    fn srid(&mut self, srid: Option<i32>) -> Result<()> {
        self.inner.srid(srid)
    }
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        let (x_transformed, y_transformed) = (self.transform_coordinates)(x, y);
        self.inner.xy(x_transformed, y_transformed, idx)
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
        let (x_transformed, y_transformed) = (self.transform_coordinates)(x, y);
        self.inner
            .coordinate(x_transformed, y_transformed, z, m, t, tm, idx)
    }
    fn empty_point(&mut self, idx: usize) -> Result<()> {
        self.inner.empty_point(idx)
    }
    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.inner.point_begin(idx)
    }
    fn point_end(&mut self, idx: usize) -> Result<()> {
        self.inner.point_end(idx)
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.inner.multipoint_begin(size, idx)
    }
    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.inner.multipoint_end(idx)
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.inner.linestring_begin(tagged, size, idx)
    }
    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.inner.linestring_end(tagged, idx)
    }
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.inner.multilinestring_begin(size, idx)
    }
    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        self.inner.multilinestring_end(idx)
    }
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.inner.polygon_begin(tagged, size, idx)
    }
    fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.inner.polygon_end(tagged, idx)
    }
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.inner.multipolygon_begin(size, idx)
    }
    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        self.inner.multipolygon_end(idx)
    }
    fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.inner.geometrycollection_begin(size, idx)
    }
    fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        self.inner.geometrycollection_end(idx)
    }
    fn circularstring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.inner.circularstring_begin(size, idx)
    }
    fn circularstring_end(&mut self, idx: usize) -> Result<()> {
        self.inner.circularstring_end(idx)
    }
    fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.inner.compoundcurve_begin(size, idx)
    }
    fn compoundcurve_end(&mut self, idx: usize) -> Result<()> {
        self.inner.compoundcurve_end(idx)
    }
    fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.inner.curvepolygon_begin(size, idx)
    }
    fn curvepolygon_end(&mut self, idx: usize) -> Result<()> {
        self.inner.curvepolygon_end(idx)
    }
    fn multicurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.inner.multicurve_begin(size, idx)
    }
    fn multicurve_end(&mut self, idx: usize) -> Result<()> {
        self.inner.multicurve_end(idx)
    }
    fn multisurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.inner.multisurface_begin(size, idx)
    }
    fn multisurface_end(&mut self, idx: usize) -> Result<()> {
        self.inner.multisurface_end(idx)
    }
    fn triangle_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.inner.triangle_begin(tagged, size, idx)
    }
    fn triangle_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.inner.triangle_end(tagged, idx)
    }
    fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.inner.polyhedralsurface_begin(size, idx)
    }
    fn polyhedralsurface_end(&mut self, idx: usize) -> Result<()> {
        self.inner.polyhedralsurface_end(idx)
    }
    fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.inner.tin_begin(size, idx)
    }
    fn tin_end(&mut self, idx: usize) -> Result<()> {
        self.inner.tin_end(idx)
    }
}

impl<T: PropertyProcessor, F: Fn(f64, f64) -> (f64, f64)> PropertyProcessor
    for WrappedProcessor<T, F>
{
    fn property(&mut self, idx: usize, name: &str, value: &ColumnValue<'_>) -> Result<bool> {
        self.inner.property(idx, name, value)
    }
}

impl<T: FeatureProcessor, F: Fn(f64, f64) -> (f64, f64)> FeatureProcessor
    for WrappedProcessor<T, F>
{
    fn dataset_begin(&mut self, name: Option<&str>) -> Result<()> {
        self.inner.dataset_begin(name)
    }
    fn dataset_end(&mut self) -> Result<()> {
        self.inner.dataset_end()
    }
    fn feature_begin(&mut self, idx: u64) -> Result<()> {
        self.inner.feature_begin(idx)
    }
    fn feature_end(&mut self, idx: u64) -> Result<()> {
        self.inner.feature_end(idx)
    }
    fn properties_begin(&mut self) -> Result<()> {
        self.inner.properties_begin()
    }
    fn properties_end(&mut self) -> Result<()> {
        self.inner.properties_end()
    }
    fn geometry_begin(&mut self) -> Result<()> {
        self.inner.geometry_begin()
    }
    fn geometry_end(&mut self) -> Result<()> {
        self.inner.geometry_end()
    }
}
