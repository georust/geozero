use geozero::error::Result;
use geozero::{CoordDimensions, GeomProcessor};
use std::io::Read;

/// Geometry processing trait.
pub trait GeozeroGeometry {
    /// Process geometry.
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()>
    where
        Self: Sized;
    /// Empty geometry.
    fn empty() -> Self
    where
        Self: Sized;
    /// Dimensions of geometry
    fn dims(&self) -> CoordDimensions {
        CoordDimensions::xy()
    }
    /// SRID of geometry
    fn srid(&self) -> Option<i32> {
        None
    }
}

/// Geometry reader trait.
pub trait GeozeroGeometryReader {
    fn read_geom<R: Read, P: GeomProcessor>(reader: R, processor: &mut P) -> Result<()>;
}
