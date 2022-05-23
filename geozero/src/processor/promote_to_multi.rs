use crate::chaining::ChainedGeomEventProcessor;
use crate::error::Result;
use crate::events::Event::*;
use crate::events::{Event, GeomEventProcessor, GeomVisitor, GeometryType};

/// Convert single geometry types to multi geometry types
pub struct PromoteToMulti;

impl ChainedGeomEventProcessor for PromoteToMulti {
    fn chain_event<P: GeomEventProcessor>(
        &mut self,
        event: Event,
        geom_type: GeometryType,
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
            LineStringBegin(size, idx) if geom_type == GeometryType::LineString => {
                visitor.multilinestring_begin(1, idx)?;
                visitor.linestring_begin(size, 0)?;
            }
            LineStringEnd(idx) if geom_type == GeometryType::LineString => {
                visitor.linestring_end(0)?;
                visitor.multilinestring_end(idx)?;
            }
            PolygonBegin(size, idx) if geom_type == GeometryType::Polygon => {
                visitor.multipolygon_begin(1, idx)?;
                visitor.polygon_begin(size, 0)?;
            }
            PolygonEnd(idx) if geom_type == GeometryType::Polygon => {
                visitor.polygon_end(0)?;
                visitor.multipolygon_end(idx)?;
            }
            _ => visitor.emit_event(event)?,
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg(all(feature = "with-geojson", feature = "with-geo"))]
mod test {
    use super::*;
    use crate::api::GeozeroGeometry;
    use crate::chaining::ChainedProcessor;
    use crate::events::GeomVisitor;
    use crate::geo_types::GeoWriter;
    use crate::geojson::conversion::ToJson;
    use crate::geojson::GeoJson;

    #[test]
    fn polygon() -> Result<()> {
        let geojson = GeoJson(
            r#"{"type": "Polygon", "coordinates": [[[20.590247,41.855404],[20.463175,41.515089],[20.605182,41.086226],[21.02004,40.842727],[20.99999,40.580004],[20.674997,40.435],[20.615,40.110007],[20.150016,39.624998],[19.98,39.694993],[19.960002,39.915006],[19.406082,40.250773],[19.319059,40.72723],[19.40355,41.409566],[19.540027,41.719986],[19.371769,41.877548],[19.304486,42.195745],[19.738051,42.688247],[19.801613,42.500093],[20.0707,42.58863],[20.283755,42.32026],[20.52295,42.21787],[20.590247,41.855404]]]}"#,
        );
        let processor1 = PromoteToMulti;
        let processor2 = GeoWriter::new(); // TODO: Json writer
        let processor = ChainedProcessor::new(processor1, processor2);
        let mut visitor = GeomVisitor::new(processor);
        geojson.process_geom(&mut visitor)?;
        let geom = visitor.processor.visitor.processor.take_geometry().unwrap(); // processor2
        let expected = r#"{"type": "MultiPolygon", "coordinates": [[[[20.590247,41.855404],[20.463175,41.515089],[20.605182,41.086226],[21.02004,40.842727],[20.99999,40.580004],[20.674997,40.435],[20.615,40.110007],[20.150016,39.624998],[19.98,39.694993],[19.960002,39.915006],[19.406082,40.250773],[19.319059,40.72723],[19.40355,41.409566],[19.540027,41.719986],[19.371769,41.877548],[19.304486,42.195745],[19.738051,42.688247],[19.801613,42.500093],[20.0707,42.58863],[20.283755,42.32026],[20.52295,42.21787],[20.590247,41.855404]]]]}"#;
        assert_eq!(expected, geom.to_json()?);
        Ok(())
    }

    #[test]
    fn geomcollection() -> Result<()> {
        let geojson = GeoJson(
            r#"{"type": "GeometryCollection", "geometries": [{"type": "Point", "coordinates": [100.1,0.1]},{"type": "LineString", "coordinates": [[101.1,0.1],[102.1,1.1]]}]}"#,
        );
        let processor1 = PromoteToMulti;
        let processor2 = GeoWriter::new(); // TODO: Json writer
        let processor = ChainedProcessor::new(processor1, processor2);
        let mut visitor = GeomVisitor::new(processor);
        geojson.process_geom(&mut visitor)?;
        let geom = visitor.processor.visitor.processor.take_geometry().unwrap(); // processor2
        let expected = r#"{"type": "GeometryCollection", "geometries": [{"type": "MultiPoint", "coordinates": [[100.1,0.1]]},{"type": "MultiLineString", "coordinates": [[[101.1,0.1],[102.1,1.1]]]}]}"#;
        assert_eq!(expected, geom.to_json()?);
        Ok(())
    }
}
