use crate::{
    error::Result, ColumnValue, CoordDimensions, FeatureProcessor, GeomProcessor, PropertyProcessor,
};

/// Wraps another [`FeatureProcessor`], first transforming coordinates.
pub struct WrappedXYProcessor<T, F: Fn(&mut f64, &mut f64)> {
    /// The underlying FeatureProcessor
    pub inner: T,
    pre_process_xy: F,
}

impl<T, F: Fn(&mut f64, &mut f64)> WrappedXYProcessor<T, F> {
    /// Wraps an inner [`FeatureProcessor`], calling `transform_coordinates` on [GeomProcessor::xy]
    /// and [GeomProcessor::coordinate] first. The function takes and returns `(x, y)`.
    pub fn new(inner: T, pre_process_xy: F) -> Self {
        Self {
            inner,
            pre_process_xy,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

// The trait has many default implementations, but every single call must be specified here to
// delegate
impl<T: GeomProcessor, F: Fn(&mut f64, &mut f64)> GeomProcessor for WrappedXYProcessor<T, F> {
    fn dimensions(&self) -> CoordDimensions {
        self.inner.dimensions()
    }
    fn multi_dim(&self) -> bool {
        self.inner.multi_dim()
    }
    fn srid(&mut self, srid: Option<i32>) -> Result<()> {
        self.inner.srid(srid)
    }
    fn xy(&mut self, mut x: f64, mut y: f64, idx: usize) -> Result<()> {
        (self.pre_process_xy)(&mut x, &mut y);
        self.inner.xy(x, y, idx)
    }
    fn coordinate(
        &mut self,
        mut x: f64,
        mut y: f64,
        z: Option<f64>,
        m: Option<f64>,
        t: Option<f64>,
        tm: Option<u64>,
        idx: usize,
    ) -> Result<()> {
        (self.pre_process_xy)(&mut x, &mut y);
        self.inner.coordinate(x, y, z, m, t, tm, idx)
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

impl<T: PropertyProcessor, F: Fn(&mut f64, &mut f64)> PropertyProcessor
    for WrappedXYProcessor<T, F>
{
    fn property(&mut self, idx: usize, name: &str, value: &ColumnValue<'_>) -> Result<bool> {
        self.inner.property(idx, name, value)
    }
}

impl<T: FeatureProcessor, F: Fn(&mut f64, &mut f64)> FeatureProcessor for WrappedXYProcessor<T, F> {
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

#[cfg(test)]
mod test {
    use crate::geo_types::GeoWriter;
    use crate::geojson::read_geojson_lines;
    use crate::GeomProcessor;
    use wkt::ToWkt;

    #[test]
    fn test_pre_process() {
        let input = r#"{ "type": "Point", "coordinates": [1.1, 1.2] }
{ "type": "Point", "coordinates": [2.1, 2.2] }
{ "type": "Point", "coordinates": [3.1, 3.2] }
"#;

        let mut geo_writer = GeoWriter::new().pre_process_xy(|x: &mut f64, y: &mut f64| {
            *x *= 2.0;
            *y *= 4.0;
        });

        read_geojson_lines(input.as_bytes(), &mut geo_writer).unwrap();
        let geometry = geo_writer.into_inner().take_geometry().unwrap();
        assert_eq!(
            geometry.wkt_string(),
            "GEOMETRYCOLLECTION(POINT(2.2 4.8),POINT(4.2 8.8),POINT(6.2 12.8))"
        );
    }
}
