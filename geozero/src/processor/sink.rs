use crate::error::Result;
use crate::events::{Event, GeomEventProcessor, GeometryType};

/// Geometry processor without any actions
pub struct GeomEventSink;

impl GeomEventProcessor for GeomEventSink {
    fn event(&mut self, _event: Event, _geom_type: GeometryType, _collection: bool) -> Result<()> {
        Ok(())
    }
}
