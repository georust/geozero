//! API traits for reading datasets and features with geomeries.
//!
//! Features are usually consumed by datasource iterators.
//! The current feature can be processed with `FeatureAccess` processing API methods.
//! Some datasources process features during consumation (e.g. reading from file).

use crate::error::{GeozeroError, Result};
use crate::feature_processor::FeatureProcessor;
use crate::property_processor::{
    PropertyProcessor, PropertyReadType, PropertyReader, PropertyReaderIdx,
};
use crate::{CoordDimensions, GeomProcessor};
use std::collections::HashMap;

/// Geometry processing trait.
pub trait GeozeroGeometry {
    /// Process geometry.
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()>
    where
        Self: Sized;
    #[deprecated(
        since = "0.7.1",
        note = "please remove implementation, not required anymore"
    )]
    #[doc(hidden)]
    fn empty() -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }
    /// Dimensions of geometry
    fn dims(&self) -> CoordDimensions {
        CoordDimensions::xy()
    }
    /// SRID of geometry
    fn srid(&self) -> Option<i32> {
        None
    }
}

/// Datasource feature consumer trait.
pub trait GeozeroDatasource {
    /// Consume and process all selected features.
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()>;
    /// Consume and process geometries of all selected features.
    fn process_geom<P: GeomProcessor>(&mut self, processor: &mut P) -> Result<()> {
        let mut geom_processor = DatasourceGeomProcessor(processor);
        self.process(&mut geom_processor)
    }
}

/// Feature processing API
pub trait FeatureAccess: FeatureProperties + GeozeroGeometry {
    /// Process feature geometries and properties.
    fn process<P: FeatureProcessor>(&self, processor: &mut P, idx: u64) -> Result<()>
    where
        Self: Sized,
    {
        processor.feature_begin(idx)?;
        processor.properties_begin()?;
        let _ = self.process_properties(processor)?;
        processor.properties_end()?;
        processor.geometry_begin()?;
        self.process_geom(processor)?;
        processor.geometry_end()?;
        processor.feature_end(idx)
    }
}

/// Feature properties processing API
pub trait FeatureProperties {
    /// Process feature properties.
    fn process_properties<P: PropertyProcessor>(&self, processor: &mut P) -> Result<bool>;
    /// Get property value by name
    fn property<T: PropertyReadType>(&self, name: &str) -> Result<T> {
        let mut reader = PropertyReader {
            name,
            value: Err(GeozeroError::ColumnUnknown),
        };
        self.process_properties(&mut reader).ok();
        reader.value
    }
    /// Get property value by number
    fn property_n<T: PropertyReadType>(&self, n: usize) -> Result<T> {
        let mut reader = PropertyReaderIdx {
            idx: n,
            value: Err(GeozeroError::ColumnUnknown),
        };
        if self.process_properties(&mut reader).is_ok() {
            reader.value
        } else {
            Err(GeozeroError::ColumnNotFound(n.to_string()))
        }
    }
    /// Return all properties in a HashMap
    /// Use `process_properties` for zero-copy access
    fn properties(&self) -> Result<HashMap<String, String>> {
        let mut properties = HashMap::new();
        let _ = self.process_properties(&mut properties)?;
        Ok(properties)
    }
}

// Newtype for GeomProcessor impl for adding no-op PropertyProcessor/FeatureProcessor impl
struct DatasourceGeomProcessor<'a, P: GeomProcessor>(&'a mut P);

// Delegate GeomProcessor impl to wrapped GeomProcessor
impl<P: GeomProcessor> GeomProcessor for DatasourceGeomProcessor<'_, P> {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        self.0.xy(x, y, idx)
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
        self.0.coordinate(x, y, z, m, t, tm, idx)
    }
    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.0.point_begin(idx)
    }
    fn point_end(&mut self, idx: usize) -> Result<()> {
        self.0.point_end(idx)
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.0.multipoint_begin(size, idx)
    }
    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.0.multipoint_end(idx)
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.0.linestring_begin(tagged, size, idx)
    }
    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.0.linestring_end(tagged, idx)
    }
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.0.multilinestring_begin(size, idx)
    }
    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        self.0.multilinestring_end(idx)
    }
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.0.polygon_begin(tagged, size, idx)
    }
    fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.0.polygon_end(tagged, idx)
    }
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.0.multipolygon_begin(size, idx)
    }
    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        self.0.multipolygon_end(idx)
    }
    fn circularstring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.0.circularstring_begin(size, idx)
    }
    fn circularstring_end(&mut self, idx: usize) -> Result<()> {
        self.0.circularstring_end(idx)
    }
    fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.0.compoundcurve_begin(size, idx)
    }
    fn compoundcurve_end(&mut self, idx: usize) -> Result<()> {
        self.0.compoundcurve_end(idx)
    }
    fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.0.curvepolygon_begin(size, idx)
    }
    fn curvepolygon_end(&mut self, idx: usize) -> Result<()> {
        self.0.curvepolygon_end(idx)
    }
    fn multicurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.0.multicurve_begin(size, idx)
    }
    fn multicurve_end(&mut self, idx: usize) -> Result<()> {
        self.0.multicurve_end(idx)
    }
    fn multisurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.0.multisurface_begin(size, idx)
    }
    fn multisurface_end(&mut self, idx: usize) -> Result<()> {
        self.0.multisurface_end(idx)
    }
    fn triangle_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.0.triangle_begin(tagged, size, idx)
    }
    fn triangle_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.0.triangle_end(tagged, idx)
    }
    fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.0.polyhedralsurface_begin(size, idx)
    }
    fn polyhedralsurface_end(&mut self, idx: usize) -> Result<()> {
        self.0.polyhedralsurface_end(idx)
    }
    fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.0.tin_begin(size, idx)
    }
    fn tin_end(&mut self, idx: usize) -> Result<()> {
        self.0.tin_end(idx)
    }
}

impl<P: GeomProcessor> PropertyProcessor for DatasourceGeomProcessor<'_, P> {}
impl<P: GeomProcessor> FeatureProcessor for DatasourceGeomProcessor<'_, P> {}
