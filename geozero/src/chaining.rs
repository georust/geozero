//! Chaining and duplexing processors.

use crate::error::Result;
use crate::events::GeomEventProcessor;
use crate::events::*;

// ------- Chain events -------

/// Processing geometry events and passing events to a chained visitor
pub trait ChainedGeomEventProcessor {
    /// Geometry processing event with geometry type information
    fn chain_event(
        &mut self,
        event: &Event,
        geom_type: GeometryType,
        collection: bool,
        visitor: &mut GeomVisitor,
    ) -> Result<()>;
}

/// Chaining [GeomEventProcessor]
pub struct ChainedProcessor<'a> {
    processor1: &'a mut dyn ChainedGeomEventProcessor,
    visitor: GeomVisitor<'a>,
}

impl<'a> ChainedProcessor<'a> {
    pub fn new(
        processor1: &'a mut dyn ChainedGeomEventProcessor,
        processor2: &'a mut dyn GeomEventProcessor,
    ) -> Self {
        ChainedProcessor {
            processor1,
            visitor: GeomVisitor::new(processor2),
        }
    }
}

impl<'a> GeomEventProcessor for ChainedProcessor<'a> {
    fn event(
        &mut self,
        event: &Event,
        geom_type: GeometryType,
        collection: bool,
    ) -> crate::error::Result<()> {
        self.processor1
            .chain_event(event, geom_type, collection, &mut self.visitor)
    }
}

// ------- Duplex ---------

/// Duplexing [GeomEventProcessor]
pub struct DuplexProcessor<'a> {
    processor1: &'a mut dyn GeomEventProcessor,
    processor2: &'a mut dyn GeomEventProcessor,
}

impl<'a> DuplexProcessor<'a> {
    pub fn new(
        processor1: &'a mut dyn GeomEventProcessor,
        processor2: &'a mut dyn GeomEventProcessor,
    ) -> Self {
        DuplexProcessor {
            processor1,
            processor2,
        }
    }
}

impl GeomEventProcessor for DuplexProcessor<'_> {
    fn event(
        &mut self,
        event: &Event,
        geom_type: GeometryType,
        collection: bool,
    ) -> crate::error::Result<()> {
        self.processor1.event(event, geom_type, collection)?;
        self.processor2.event(event, geom_type, collection)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::Result;
    use crate::events::test::{GeomEventBuffer, NullIsland};
    use crate::events::Event::*;
    use crate::processor::GeomEventSink;

    pub struct PromoteToMulti;
    impl ChainedGeomEventProcessor for PromoteToMulti {
        fn chain_event(
            &mut self,
            event: &Event,
            _geom_type: GeometryType,
            _collection: bool,
            visitor: &mut GeomVisitor,
        ) -> Result<()> {
            match *event {
                PointBegin(idx) => {
                    visitor.multipoint_begin(1, idx)?;
                }
                PointEnd(idx) => {
                    visitor.multipoint_end(idx)?;
                }
                _ => visitor.emit_event(event)?,
            }
            Ok(())
        }
    }

    #[test]
    fn chained_processor() -> Result<()> {
        let mut buffer_processor = GeomEventBuffer::new();
        let mut multi = PromoteToMulti;
        let mut processor = ChainedProcessor::new(&mut multi, &mut buffer_processor);
        let mut visitor = GeomVisitor::new(&mut processor);

        let mut geom = NullIsland;
        geom.process_geom(&mut visitor)?;

        assert_eq!(
            buffer_processor.buffer,
            [MultiPointBegin(1, 0), Xy(0.0, 0.0, 0), MultiPointEnd(0)]
        );

        Ok(())
    }

    #[test]
    fn duplex_processor() -> Result<()> {
        let mut buffer_processor = GeomEventBuffer::new();
        let mut sink = GeomEventSink;
        let mut processor = DuplexProcessor::new(&mut sink, &mut buffer_processor);
        let mut visitor = GeomVisitor::new(&mut processor);

        let mut geom = NullIsland;
        geom.process_geom(&mut visitor)?;

        assert_eq!(
            buffer_processor.buffer,
            [PointBegin(0), Xy(0.0, 0.0, 0), PointEnd(0),]
        );

        Ok(())
    }

    #[test]
    fn chain_and_duplex() -> Result<()> {
        // geom  ------+----> PromoteToMulti (1a) ----> GeomEventBuffer (2a)
        //             |
        //             +----> GeomEventBuffer (1b)
        let mut processor1a = PromoteToMulti;
        let mut processor2a = GeomEventBuffer::new();
        let mut processor1b = GeomEventBuffer::new();
        let mut processor_a = ChainedProcessor::new(&mut processor1a, &mut processor2a);
        let mut processor = DuplexProcessor::new(&mut processor_a, &mut processor1b);
        let mut visitor = GeomVisitor::new(&mut processor);

        let mut geom = NullIsland;
        geom.process_geom(&mut visitor)?;

        assert_eq!(
            processor2a.buffer,
            [MultiPointBegin(1, 0), Xy(0.0, 0.0, 0), MultiPointEnd(0)]
        );
        assert_eq!(
            processor1b.buffer,
            [PointBegin(0), Xy(0.0, 0.0, 0), PointEnd(0),]
        );

        Ok(())
    }
}
