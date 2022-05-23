//! Chaining and duplexing processors.

use crate::error::Result;
use crate::events::GeomEventProcessor;
use crate::events::*;

// ------- Chain events -------

/// Processing geometry events and passing events to a chained visitor
pub trait ChainedGeomEventProcessor {
    /// Geometry processing event with geometry type information
    fn chain_event<P: GeomEventProcessor>(
        &mut self,
        event: Event,
        geom_type: GeometryType,
        collection: bool,
        visitor: &mut GeomVisitor<P>,
    ) -> Result<()>;
}

/// Chaining [GeomEventProcessor]
pub struct ChainedProcessor<P1: ChainedGeomEventProcessor, P2: GeomEventProcessor> {
    pub processor1: P1,
    pub visitor: GeomVisitor<P2>,
}

impl<'a, P1: ChainedGeomEventProcessor, P2: GeomEventProcessor> ChainedProcessor<P1, P2> {
    pub fn new(processor1: P1, processor2: P2) -> Self {
        ChainedProcessor {
            processor1,
            visitor: GeomVisitor::new(processor2),
        }
    }
}

impl<'a, P1: ChainedGeomEventProcessor, P2: GeomEventProcessor> GeomEventProcessor
    for ChainedProcessor<P1, P2>
{
    fn event(
        &mut self,
        event: Event,
        geom_type: GeometryType,
        collection: bool,
    ) -> crate::error::Result<()> {
        self.processor1
            .chain_event(event, geom_type, collection, &mut self.visitor)
    }
}

// ------- Duplex ---------

/// Duplexing [GeomEventProcessor]
pub struct DuplexProcessor<P1: GeomEventProcessor, P2: GeomEventProcessor> {
    pub processor1: P1,
    pub processor2: P2,
}

impl<P1: GeomEventProcessor, P2: GeomEventProcessor> DuplexProcessor<P1, P2> {
    pub fn new(processor1: P1, processor2: P2) -> Self {
        DuplexProcessor {
            processor1,
            processor2,
        }
    }
}

impl<P1: GeomEventProcessor, P2: GeomEventProcessor> GeomEventProcessor
    for DuplexProcessor<P1, P2>
{
    fn event(
        &mut self,
        event: Event,
        geom_type: GeometryType,
        collection: bool,
    ) -> crate::error::Result<()> {
        self.processor1
            .event(event.clone(), geom_type, collection)?;
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
        fn chain_event<P: GeomEventProcessor>(
            &mut self,
            event: Event,
            _geom_type: GeometryType,
            _collection: bool,
            visitor: &mut GeomVisitor<P>,
        ) -> Result<()> {
            match event {
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
        let processor = ChainedProcessor::new(PromoteToMulti, GeomEventBuffer::new());
        let mut visitor = GeomVisitor::new(processor);

        let mut geom = NullIsland;
        geom.process_geom(&mut visitor)?;

        assert_eq!(
            visitor.processor.visitor.processor.buffer,
            [MultiPointBegin(1, 0), Xy(0.0, 0.0, 0), MultiPointEnd(0)]
        );

        Ok(())
    }

    #[test]
    fn duplex_processor() -> Result<()> {
        let processor = DuplexProcessor::new(GeomEventSink, GeomEventBuffer::new());
        let mut visitor = GeomVisitor::new(processor);

        let mut geom = NullIsland;
        geom.process_geom(&mut visitor)?;

        assert_eq!(
            visitor.processor.processor2.buffer,
            [PointBegin(0), Xy(0.0, 0.0, 0), PointEnd(0),]
        );

        Ok(())
    }

    #[test]
    fn chain_and_duplex() -> Result<()> {
        // geom  ------+----> PromoteToMulti (1a) ----> GeomEventBuffer (2a)
        //             |
        //             +----> GeomEventBuffer (1b)
        let processor_a = ChainedProcessor::new(PromoteToMulti, GeomEventBuffer::new());
        let processor = DuplexProcessor::new(processor_a, GeomEventBuffer::new());
        let mut visitor = GeomVisitor::new(processor);

        let mut geom = NullIsland;
        geom.process_geom(&mut visitor)?;

        assert_eq!(
            visitor.processor.processor1.visitor.processor.buffer, // 2a
            [MultiPointBegin(1, 0), Xy(0.0, 0.0, 0), MultiPointEnd(0)]
        );
        assert_eq!(
            visitor.processor.processor2.buffer, // 1b
            [PointBegin(0), Xy(0.0, 0.0, 0), PointEnd(0),]
        );

        Ok(())
    }
}
