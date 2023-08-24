use crate::{
    error::{GeozeroError, Result},
    FeatureProcessor, GeomProcessor, GeozeroDatasource, GeozeroGeometry,
};

use super::{process_geojson_geom_n, process_properties};

use std::io::{BufRead, BufReader, Read};

use geojson::{Feature, GeoJson as GeoGeoJson, Geometry};

/// Line Delimited GeoJSON Reader: One feature per line.
///
/// See <https://jsonlines.org>
pub struct GeoJsonLineReader<R: Read>(pub(crate) R);
impl<R: Read> GeoJsonLineReader<R> {
    pub fn new(read: R) -> Self {
        Self(read)
    }
}

impl<R: Read + Clone> GeozeroGeometry for GeoJsonLineReader<R> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()>
    where
        Self: Sized,
    {
        read_geojson_line_geometries(&mut self.0.clone(), processor)
    }
}

impl<R: Read> GeozeroDatasource for GeoJsonLineReader<R> {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        read_geojson_lines(&mut self.0, processor)
    }
}

/// Read and process line delimited GeoJSON (one Geometry per line).
pub fn read_geojson_line_geometries(
    reader: impl Read,
    processor: &mut impl GeomProcessor,
) -> Result<()> {
    let buf_reader = BufReader::new(reader);

    let mut started = false;
    let mut add_geometry_to_collection = |idx: usize, geometry: &Geometry| {
        if !started {
            started = true;

            // We don't know how many lines are in the file, so we dont' know the size of the geometry collection,
            // but at this point we *do* know that it's non-zero. Currently there aren't any other significant
            // distinctions for knowing collection size.
            //
            // If we wanted to get this more exactly, we'd have to take multiple passes on the file or
            // hold the whole thing in memory, which doesn't seem worth it.
            processor.geometrycollection_begin(1, 0)?;
        }
        process_geometry(processor, idx, geometry)
    };

    for (idx, line) in buf_reader.lines().enumerate() {
        match line?.parse::<GeoGeoJson>()? {
            GeoGeoJson::Feature(feature) => {
                if let Some(geometry) = feature.geometry {
                    add_geometry_to_collection(idx, &geometry)?;
                }
            }
            GeoGeoJson::Geometry(geometry) => add_geometry_to_collection(idx, &geometry)?,
            _ => {
                return Err(GeozeroError::Dataset("line-delimited GeoJson ('geojsonl') files must have one Feature or Geometry per line".to_string()));
            }
        };
    }

    if !started {
        // No geometries were added, so it's an empty collection.
        processor.geometrycollection_begin(0, 0)?;
    }
    processor.geometrycollection_end(0)?;
    Ok(())
}

/// Read and process line delimited GeoJSON (one object per line).
pub fn read_geojson_lines(reader: impl Read, processor: &mut impl FeatureProcessor) -> Result<()> {
    let buf_reader = BufReader::new(reader);

    processor.dataset_begin(None)?;
    for (idx, line) in buf_reader.lines().enumerate() {
        match line?.parse::<GeoGeoJson>()? {
            GeoGeoJson::Feature(feature) => process_feature(processor, idx, &feature)?,
            GeoGeoJson::Geometry(geometry) => process_geometry(processor, idx, &geometry)?,
            _ => {
                return Err(GeozeroError::Dataset("line-delimited GeoJson ('geojsonl') files must have one Feature or Geometry per line".to_string()));
            }
        }
    }
    processor.dataset_end()
}

fn process_feature(
    processor: &mut impl FeatureProcessor,
    idx: usize,
    feature: &Feature,
) -> Result<()> {
    processor.feature_begin(idx as u64)?;
    if let Some(ref properties) = feature.properties {
        processor.properties_begin()?;
        process_properties(properties, processor)?;
        processor.properties_end()?;
    }
    if let Some(ref geometry) = feature.geometry {
        processor.geometry_begin()?;
        process_geometry(processor, 0, geometry)?;
        processor.geometry_end()?;
    }
    processor.feature_end(idx as u64)?;
    Ok(())
}

fn process_geometry(
    processor: &mut impl GeomProcessor,
    idx: usize,
    geometry: &Geometry,
) -> Result<()> {
    process_geojson_geom_n(geometry, idx, processor)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProcessToJson;
    use crate::ToWkt;

    #[test]
    fn good_geometries() {
        let input = r#"{ "type": "Point", "coordinates": [1.1, 1.2] }
{ "type": "Point", "coordinates": [2.1, 2.2] }
{ "type": "Point", "coordinates": [3.1, 3.2] }
"#;
        let reader = GeoJsonLineReader(input.as_bytes());
        let wkt = reader.to_wkt().unwrap();
        let expected = "GEOMETRYCOLLECTION(POINT(1.1 1.2),POINT(2.1 2.2),POINT(3.1 3.2))";
        assert_eq!(wkt, expected);
    }

    #[test]
    fn good_features() {
        let input = r#"{ "type": "Feature", "geometry": { "type": "Point", "coordinates": [1.1, 1.2] }, "properties": { "name": "first" } }
{ "type": "Feature", "geometry": { "type": "Point", "coordinates": [2.1, 2.2] }, "properties": { "name": "second" } }
{ "type": "Feature", "geometry": { "type": "Point", "coordinates": [3.1, 3.3] }, "properties": { "name": "third" } }
"#;
        let mut reader = GeoJsonLineReader(input.as_bytes());
        let json_string = reader.to_json().unwrap();
        let json: serde_json::Value = serde_json::from_str(&json_string)
            .unwrap_or_else(|err| panic!("invalid json: `{json_string}`: {err}"));
        let expected = serde_json::json!({
            "type": "FeatureCollection",
            "features": [
                {"type": "Feature", "properties": {"name": "first"}, "geometry": {"type": "Point", "coordinates": [1.1,1.2]}},
                {"type": "Feature", "properties": {"name": "second"}, "geometry": {"type": "Point", "coordinates": [2.1,2.2]}},
                {"type": "Feature", "properties": {"name": "third"}, "geometry": {"type": "Point", "coordinates": [3.1,3.3]}}]
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn malformed_json() {
        let input = r#"{ "type": "Feature", "geometry": { "type": "Point", "coordinates": [1.1, 1.2] }, "properties": { "name": "first" } }
ooops this is malformed json { "type": "Feature", "geometry": { "type": "Point", "coordinates": [2.1, 2.2] }, "properties": { "name": "second" } }
{ "type": "Feature", "geometry": { "type": "Point", "coordinates": [3.1, 3.3] }, "properties": { "name": "third" } }
"#;
        let mut reader = GeoJsonLineReader(input.as_bytes());
        _ = reader.to_json().unwrap_err();
    }
    #[test]
    fn valid_json_but_not_one_feature_per_line() {
        let input = r#"{
            "type": "Feature",
            "geometry": {
                "type": "Point",
                "coordinates": [1.1, 1.2]
            }, "
            properties": { "name": "first" }
        }"#;
        let mut reader = GeoJsonLineReader(input.as_bytes());
        _ = reader.to_json().unwrap_err();
    }
}
