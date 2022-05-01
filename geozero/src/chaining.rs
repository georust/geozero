//! Chaining and duplexing processors.

use crate::error::Result;
use crate::events::GeomEventProcessor;
use crate::events::*;

// ------- Chain events -------

/// Processing geometry events and passing events to a chained visitor
pub trait CĥainedGeomEventProcessor {
    /// Geometry processing event with geometry type information
    fn chain_event<P: GeomEventProcessor>(
        &mut self,
        event: Event,
        geom_type: GeometryType,
        collection: bool,
        visitor: &mut GeomVisitor<'_, P>,
    ) -> Result<()>;
}

/// Chaining [GeomEventProcessor]
pub struct ChainedProcessor<'a, P1: CĥainedGeomEventProcessor, P2: GeomEventProcessor> {
    processor1: &'a mut P1,
    visitor: GeomVisitor<'a, P2>,
}

impl<'a, P1: CĥainedGeomEventProcessor, P2: GeomEventProcessor> ChainedProcessor<'a, P1, P2> {
    pub fn new(processor1: &'a mut P1, processor2: &'a mut P2) -> Self {
        ChainedProcessor {
            processor1,
            visitor: GeomVisitor::new(processor2),
        }
    }
}

impl<'a, P1: CĥainedGeomEventProcessor, P2: GeomEventProcessor> GeomEventProcessor
    for ChainedProcessor<'a, P1, P2>
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
pub struct DuplexProcessor<'a, P1: GeomEventProcessor, P2: GeomEventProcessor> {
    processor1: &'a mut P1,
    processor2: &'a mut P2,
}

impl<'a, P1: GeomEventProcessor, P2: GeomEventProcessor> DuplexProcessor<'a, P1, P2> {
    pub fn new(processor1: &'a mut P1, processor2: &'a mut P2) -> Self {
        DuplexProcessor {
            processor1,
            processor2,
        }
    }
}

impl<'a, P1: GeomEventProcessor, P2: GeomEventProcessor> GeomEventProcessor
    for DuplexProcessor<'a, P1, P2>
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
    use crate::events::GeomEventSink;

    pub struct PromoteToMulti;
    impl CĥainedGeomEventProcessor for PromoteToMulti {
        fn chain_event<P: GeomEventProcessor>(
            &mut self,
            event: Event,
            geom_type: GeometryType,
            _collection: bool,
            visitor: &mut GeomVisitor<'_, P>,
        ) -> Result<()> {
            match event {
                PointBegin(idx) if geom_type == GeometryType::Point => {
                    visitor.multipoint_begin(1, idx)?;
                    visitor.point_begin(0)?;
                }
                PointEnd(idx) if geom_type == GeometryType::Point => {
                    visitor.point_end(0)?;
                    visitor.multipoint_end(idx)?;
                }
                _ => visitor.emit(event)?,
            }
            Ok(())
        }
    }

    #[test]
    fn chained_processor() -> Result<()> {
        let mut processor1 = PromoteToMulti;
        let mut processor2 = GeomEventBuffer::new();
        let mut processor = ChainedProcessor::new(&mut processor1, &mut processor2);
        let mut visitor = GeomVisitor::new(&mut processor);

        let mut geom = NullIsland;
        geom.process_geom(&mut visitor)?;

        assert_eq!(
            processor2.buffer,
            [
                MultiPointBegin(1, 0),
                PointBegin(0),
                Xy(0.0, 0.0, 0),
                PointEnd(0),
                MultiPointEnd(0)
            ]
        );

        Ok(())
    }

    #[test]
    fn duplex_processor() -> Result<()> {
        let mut processor1 = GeomEventSink;
        let mut processor2 = GeomEventBuffer::new();
        let mut processor = DuplexProcessor::new(&mut processor1, &mut processor2);
        let mut visitor = GeomVisitor::new(&mut processor);

        let mut geom = NullIsland;
        geom.process_geom(&mut visitor)?;

        assert_eq!(
            processor2.buffer,
            [PointBegin(0), Xy(0.0, 0.0, 0), PointEnd(0),]
        );

        Ok(())
    }

    #[test]
    fn chain_and_duplex() -> Result<()> {
        // geom  ------+----> PromotToMulti (1a) ----> GeomEventBuffer (2a)
        //             |
        //             +----> GeomEventBuffer (1b)
        let mut processor1a = PromoteToMulti;
        let mut processor2a = GeomEventBuffer::new();
        let mut processor_a = ChainedProcessor::new(&mut processor1a, &mut processor2a);
        let mut processor1b = GeomEventBuffer::new();
        let mut processor = DuplexProcessor::new(&mut processor_a, &mut processor1b);
        let mut visitor = GeomVisitor::new(&mut processor);

        let mut geom = NullIsland;
        geom.process_geom(&mut visitor)?;

        assert_eq!(
            processor2a.buffer,
            [
                MultiPointBegin(1, 0),
                PointBegin(0),
                Xy(0.0, 0.0, 0),
                PointEnd(0),
                MultiPointEnd(0)
            ]
        );
        assert_eq!(
            processor1b.buffer,
            [PointBegin(0), Xy(0.0, 0.0, 0), PointEnd(0),]
        );

        Ok(())
    }
}
