use crate::chaining::ChainedGeomEventProcessor;
use crate::error::Result;
use crate::events::Event::*;
use crate::events::{Event, GeomEventProcessor, GeomVisitor, GeometryType};
use std::f64::consts;

/// Convert lon/lat to Spherical Mercator in meters
pub struct LonLatToMercator;

fn lonlat_to_merc(lon: f64, lat: f64) -> (f64, f64) {
    let x = 6378137.0 * lon.to_radians();
    let y = 6378137.0 * ((consts::PI * 0.25) + (0.5 * lat.to_radians())).tan().ln();
    (x, y)
}

impl ChainedGeomEventProcessor for LonLatToMercator {
    fn chain_event<P: GeomEventProcessor>(
        &mut self,
        event: &Event,
        geom_type: GeometryType,
        collection: bool,
        visitor: &mut GeomVisitor<P>,
    ) -> Result<()> {
        match *event {
            Xy(x, y, idx) => {
                let (x, y) = lonlat_to_merc(x, y);
                visitor.chain_event(&Xy(x, y, idx), geom_type, collection)?;
            }
            Coordinate(x, y, z, m, t, tm, idx) => {
                let (x, y) = lonlat_to_merc(x, y);
                visitor.chain_event(&Coordinate(x, y, z, m, t, tm, idx), geom_type, collection)?;
            }
            _ => visitor.chain_event(event, geom_type, collection)?,
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
        let processor1 = LonLatToMercator;
        let processor2 = GeoWriter::new(); // TODO: Json writer
        let processor = ChainedProcessor::new(processor1, processor2);
        let mut visitor = GeomVisitor::new(processor);
        geojson.process_geom(&mut visitor)?;
        let geom = visitor.processor.visitor.processor.take_geometry().unwrap();
        let expected = r#"{"type": "Polygon", "coordinates": [[[2292095.811347729,5139344.213425913],[2277950.2210136456,5088616.6330585545],[2293758.3679427262,5025068.310371113],[2339940.1492542424,4989171.535691388],[2337708.193463837,4950588.339806374],[2301530.138192459,4929358.117761691],[2294851.3027033345,4881941.097754033],[2243089.5205963156,4811596.717986056],[2224163.426049606,4821717.98297803],[2221937.2588727223,4853598.859120461],[2160275.1665325123,4902451.127115252],[2150587.810485209,4972191.022999316],[2159993.3055818235,5072941.548683907],[2175185.8557268167,5119126.5562673],[2156455.4608449223,5142654.340537157],[2148965.551545879,5190346.344385106],[2197229.7865716643,5264639.607013478],[2204305.476045466,5236187.825747299],[2234260.1038645557,5249565.284016839],[2257977.2779755164,5209074.219280535],[2284604.3435758143,5193671.390117393],[2292095.811347729,5139344.213425913]]]}"#;
        assert_eq!(expected, geom.to_json()?);
        Ok(())
    }
}
