use crate::error::{GeozeroError, Result};
use crate::{ColumnValue, FeatureProcessor, GeomProcessor, GeozeroDatasource, GeozeroGeometry};

use std::io::Read;
use std::str::FromStr;

pub struct Csv<'a> {
    csv_text: &'a str,
    geometry_column_name: String,
}

impl<'a> Csv<'a> {
    pub fn new(geometry_column_name: &str, csv_text: &'a str) -> Self {
        Self {
            csv_text,
            geometry_column_name: geometry_column_name.to_string(),
        }
    }
}

impl GeozeroDatasource for Csv<'_> {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        process_csv_features(
            self.csv_text.as_bytes(),
            processor,
            &self.geometry_column_name,
        )
    }
}

impl GeozeroGeometry for Csv<'_> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        process_csv_geom(
            self.csv_text.as_bytes(),
            processor,
            &self.geometry_column_name,
        )
    }
}

pub struct CsvString {
    csv_text: String,
    geometry_column_name: String,
}

impl CsvString {
    pub fn new(geometry_column_name: &str, csv_text: String) -> Self {
        Self {
            csv_text,
            geometry_column_name: geometry_column_name.to_string(),
        }
    }
}

impl GeozeroDatasource for CsvString {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        process_csv_features(
            self.csv_text.as_bytes(),
            processor,
            &self.geometry_column_name,
        )
    }
}

impl GeozeroGeometry for CsvString {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        process_csv_geom(
            self.csv_text.as_bytes(),
            processor,
            &self.geometry_column_name,
        )
    }
}

pub struct CsvReader<R: Read> {
    inner: R,
    geometry_column_name: String,
}

impl<R: Read> CsvReader<R> {
    pub fn new(geometry_column_name: &str, inner: R) -> Self {
        Self {
            inner,
            geometry_column_name: geometry_column_name.to_string(),
        }
    }
}

impl<R: Read> GeozeroDatasource for CsvReader<R> {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        process_csv_features(&mut self.inner, processor, &self.geometry_column_name)
    }
}

impl<R: Read + Clone> GeozeroGeometry for CsvReader<R> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        process_csv_geom(self.inner.clone(), processor, &self.geometry_column_name)
    }
}

pub fn process_csv_geom(
    input: impl Read,
    processor: &mut impl GeomProcessor,
    geometry_column: &str,
) -> Result<()> {
    let mut reader = csv::Reader::from_reader(input);
    let headers = reader.headers()?.clone();

    let geometry_idx = headers
        .iter()
        .position(|f| f == geometry_column)
        .ok_or(GeozeroError::ColumnNotFound)?;

    let mut collection_started = false;

    for (record_idx, record) in reader.into_records().enumerate() {
        let record = record?;
        let geometry_field = record
            .get(geometry_idx)
            .ok_or(GeozeroError::ColumnNotFound)?;
        let wkt = wkt::Wkt::from_str(geometry_field)
            .map_err(|e| GeozeroError::Geometry(e.to_string()))?;

        // We don't know how many lines are in the file, so we dont' know the size of the geometry collection,
        // but at this point we *do* know that it's non-zero. Currently there aren't any other significant
        // distinctions for knowing collection size.
        //
        // If we wanted to get this more exactly, we'd have to take multiple passes on the file or
        // hold the whole thing in memory, which doesn't seem worth it.
        if !collection_started {
            collection_started = true;
            processor.geometrycollection_begin(1, 0)?;
        }

        crate::wkt::wkt_reader::process_wkt_geom_n(&wkt.item, record_idx, processor).map_err(
            |e| {
                // +2 to start at line 1 and to account for the header row
                let line = record_idx + 2;
                log::warn!("line {line}: invalid WKT: '{geometry_field}', record: {record:?}");
                e
            },
        )?;
    }

    if !collection_started {
        // If collection hasn't been started at this point, it's empty.
        processor.geometrycollection_begin(0, 0)?;
    }
    processor.geometrycollection_end(0)
}

pub fn process_csv_features(
    input: impl Read,
    processor: &mut impl FeatureProcessor,
    geometry_column: &str,
) -> Result<()> {
    let mut reader = csv::Reader::from_reader(input);
    let headers = reader.headers()?.clone();
    processor.dataset_begin(None)?;

    let geometry_idx = headers
        .iter()
        .position(|f| f == geometry_column)
        .ok_or(GeozeroError::ColumnNotFound)?;

    for (feature_idx, record) in reader.into_records().enumerate() {
        let record = record?;
        processor.feature_begin(feature_idx as u64)?;

        processor.properties_begin()?;

        let properties_iter = headers
            .iter()
            .zip(record.iter())
            .enumerate()
            // skip the geometry field -  we process it after all the "properties"
            .filter(|(input_idx, _)| *input_idx != geometry_idx)
            .map(|(_input_idx, (header, value))| (header, value));

        for (output_idx, (header, field)) in properties_iter.enumerate() {
            let value = &ColumnValue::String(field);
            processor.property(output_idx, header, value)?;
        }

        processor.properties_end()?;

        let geometry_field = record
            .get(geometry_idx)
            .ok_or(GeozeroError::ColumnNotFound)?;

        // Do all formats allow empty geometries?
        if !geometry_field.is_empty() {
            processor.geometry_begin()?;
            crate::wkt::wkt_reader::read_wkt(&mut geometry_field.as_bytes(), processor).map_err(
                |e| {
                    // +2 to start at line 1 and to account for the header row
                    let line = feature_idx + 2;
                    log::warn!("line {line}: invalid WKT: '{geometry_field}', record: {record:?}");
                    e
                },
            )?;
            processor.geometry_end()?;
        }

        processor.feature_end(feature_idx as u64)?;
    }

    processor.dataset_end()
}

impl From<csv::Error> for GeozeroError {
    fn from(error: csv::Error) -> Self {
        if matches!(error.kind(), csv::ErrorKind::Io(_)) {
            match error.into_kind() {
                csv::ErrorKind::Io(io_err) => GeozeroError::IoError(io_err),
                _ => unreachable!(),
            }
        } else {
            GeozeroError::Dataset(error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_feature_processor() {
        use crate::geojson::conversion::ProcessToJson;

        let mut csv = Csv::new(
            "report location",
            r#"address,type,datetime,report location,incident number
904 7th Av,Car Fire,05/22/2019 12:55:00 PM,POINT (-122.329051 47.6069),F190051945
9610 53rd Av S,Aid Response,05/22/2019 12:55:00 PM,POINT (-122.266529 47.515984),F190051946"#,
        );

        let expected_geojson = serde_json::json!({
            "type": "FeatureCollection",
            "features": [
               {
                   "type": "Feature",
                   "geometry": {
                       "type": "Point",
                       "coordinates": [-122.329051, 47.6069]
                   },
                   "properties": {
                       "address": "904 7th Av",
                       "type": "Car Fire",
                       "datetime": "05/22/2019 12:55:00 PM",
                       "incident number": "F190051945"
                   }
               },
               {
                   "type": "Feature",
                   "geometry": {
                       "type": "Point",
                       "coordinates": [-122.266529, 47.515984]
                   },
                   "properties": {
                       "address": "9610 53rd Av S",
                       "type": "Aid Response",
                       "datetime": "05/22/2019 12:55:00 PM",
                       "incident number": "F190051946"
                   }
               }
            ]
        });

        let actual_geojson = csv.to_json().unwrap();
        let actual_geojson: serde_json::Value = serde_json::from_str(&actual_geojson).unwrap();

        assert_eq!(expected_geojson, actual_geojson)
    }
    #[test]
    fn csv_string_feature_processor() {
        use crate::geojson::conversion::ProcessToJson;

        let mut csv = CsvString::new(
            "report location",
            r#"address,type,datetime,report location,incident number
904 7th Av,Car Fire,05/22/2019 12:55:00 PM,POINT (-122.329051 47.6069),F190051945
9610 53rd Av S,Aid Response,05/22/2019 12:55:00 PM,POINT (-122.266529 47.515984),F190051946"#
                .to_string(),
        );
        let expected_geojson = serde_json::json!({
            "type": "FeatureCollection",
            "features": [
               {
                   "type": "Feature",
                   "geometry": {
                       "type": "Point",
                       "coordinates": [-122.329051, 47.6069]
                   },
                   "properties": {
                       "address": "904 7th Av",
                       "type": "Car Fire",
                       "datetime": "05/22/2019 12:55:00 PM",
                       "incident number": "F190051945"
                   }
               },
               {
                   "type": "Feature",
                   "geometry": {
                       "type": "Point",
                       "coordinates": [-122.266529, 47.515984]
                   },
                   "properties": {
                       "address": "9610 53rd Av S",
                       "type": "Aid Response",
                       "datetime": "05/22/2019 12:55:00 PM",
                       "incident number": "F190051946"
                   }
               }
            ]
        });

        let actual_geojson = csv.to_json().unwrap();
        let actual_geojson: serde_json::Value = serde_json::from_str(&actual_geojson).unwrap();

        assert_eq!(expected_geojson, actual_geojson)
    }

    #[test]
    fn csv_reader_feature_processor() {
        use crate::geojson::conversion::ProcessToJson;

        let mut csv = CsvReader::new(
            "report location",
            r#"address,type,datetime,report location,incident number
904 7th Av,Car Fire,05/22/2019 12:55:00 PM,POINT (-122.329051 47.6069),F190051945
9610 53rd Av S,Aid Response,05/22/2019 12:55:00 PM,POINT (-122.266529 47.515984),F190051946"#
                .as_bytes(),
        );
        let expected_geojson = serde_json::json!({
            "type": "FeatureCollection",
            "features": [
               {
                   "type": "Feature",
                   "geometry": {
                       "type": "Point",
                       "coordinates": [-122.329051, 47.6069]
                   },
                   "properties": {
                       "address": "904 7th Av",
                       "type": "Car Fire",
                       "datetime": "05/22/2019 12:55:00 PM",
                       "incident number": "F190051945"
                   }
               },
               {
                   "type": "Feature",
                   "geometry": {
                       "type": "Point",
                       "coordinates": [-122.266529, 47.515984]
                   },
                   "properties": {
                       "address": "9610 53rd Av S",
                       "type": "Aid Response",
                       "datetime": "05/22/2019 12:55:00 PM",
                       "incident number": "F190051946"
                   }
               }
            ]
        });

        let actual_geojson = csv.to_json().unwrap();
        let actual_geojson: serde_json::Value = serde_json::from_str(&actual_geojson).unwrap();

        assert_eq!(expected_geojson, actual_geojson)
    }

    #[test]
    fn geom_processor() {
        use crate::geojson::conversion::ToJson;

        let csv = Csv::new(
            "report location",
            r#"address,type,datetime,report location,incident number
904 7th Av,Car Fire,05/22/2019 12:55:00 PM,POINT (-122.329051 47.6069),F190051945
9610 53rd Av S,Aid Response,05/22/2019 12:55:00 PM,POINT (-122.266529 47.515984),F190051946"#,
        );

        let expected_geojson = serde_json::json!({
            "type": "GeometryCollection",
            "geometries": [
                {
                    "type": "Point",
                    "coordinates": [-122.329051,47.6069]
                },
                {
                    "type": "Point",
                    "coordinates": [-122.266529,47.515984]
                }
            ]
        });

        let actual_geojson = csv.to_json().unwrap();
        let actual_geojson: serde_json::Value = serde_json::from_str(&actual_geojson).unwrap();

        assert_eq!(expected_geojson, actual_geojson);
    }

    #[test]
    fn missing_geom() {
        use crate::geojson::conversion::ProcessToJson;

        let mut csv = Csv::new(
            "report location",
            r#"address,type,datetime,report location,incident number
904 7th Av,Car Fire,05/22/2019 12:55:00 PM,POINT (-122.329051 47.6069),F190051945
9610 53rd Av S,Aid Response,05/22/2019 12:55:00 PM,,F190051946"#,
        );

        let json = csv.to_json().unwrap();

        // ensure the json is parsable. Note it's not parsable back as geojson due to the
        // missing geometry. Some formats, like FGB, will tolerate this null geometry.
        serde_json::from_str::<serde_json::Value>(&json).unwrap();
    }

    #[test]
    fn non_empty_geometry_collection() {
        use crate::ToWkt;

        let input = r#"address,type,datetime,report location,incident number
904 7th Av,Car Fire,05/22/2019 12:55:00 PM,POINT (-122.329051 47.6069),F190051945
9610 53rd Av S,Aid Response,05/22/2019 12:55:00 PM,POINT (-122.266529 47.515984),F190051946
"#;
        let csv = CsvReader::new("report location", input.as_bytes());

        let actual = csv.to_wkt().unwrap();

        let expected =
            "GEOMETRYCOLLECTION(POINT(-122.329051 47.6069),POINT(-122.266529 47.515984))";
        assert_eq!(expected, actual);
    }

    #[test]
    fn empty_geometry_collection() {
        use crate::ToWkt;

        let input = r#"address,type,datetime,report location,incident number
"#;
        let csv = CsvReader::new("report location", input.as_bytes());

        let actual = csv.to_wkt().unwrap();

        let expected = "GEOMETRYCOLLECTION EMPTY";
        assert_eq!(expected, actual);
    }
}
