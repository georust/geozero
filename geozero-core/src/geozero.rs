use geozero::error::Result;
use geozero::GeomProcessor;

/// Geometry processing trait.
pub trait GeozeroGeometry {
    /// Process geometry.
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()>
    where
        Self: Sized;
}
