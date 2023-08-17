use crate::error::Result;
use crate::wkt::WktWriter;
use crate::{ColumnValue, CoordDimensions, FeatureProcessor, GeomProcessor, PropertyProcessor};

use std::io::Write;

pub struct CsvWriter<W: Write> {
    csv: csv::Writer<W>,
    headers: Vec<String>,
    has_written_first_record: bool,
    current_row_props: Vec<String>,
    wkt_writer: WktWriter<Vec<u8>>,
}

impl<W: Write> CsvWriter<W> {
    pub fn new(out: W) -> Self {
        Self::with_dims(out, CoordDimensions::default())
    }

    pub fn with_dims(out: W, dims: CoordDimensions) -> Self {
        Self {
            csv: csv::Writer::from_writer(out),
            headers: vec!["geometry".to_string()],
            has_written_first_record: false,
            current_row_props: vec![],
            wkt_writer: WktWriter::with_dims(vec![], dims),
        }
    }

    fn has_started_writing_geometry_in_this_row(&self) -> bool {
        !self.wkt_writer.out.is_empty()
    }

    fn offset_geom_idx(&self, input_idx: usize) -> usize {
        if self.has_started_writing_geometry_in_this_row() {
            input_idx
        } else {
            // avoid prefixing subsequent row geometry with a comma
            0
        }
    }
}

impl<W: Write> FeatureProcessor for CsvWriter<W> {
    fn dataset_begin(&mut self, _name: Option<&str>) -> Result<()> {
        debug_assert_eq!(self.headers, &["geometry"]);
        Ok(())
    }
    fn dataset_end(&mut self) -> Result<()> {
        self.headers = vec!["geometry".to_string()];
        Ok(())
    }
    fn feature_begin(&mut self, _idx: u64) -> Result<()> {
        debug_assert!(self.current_row_props.is_empty());
        Ok(())
    }

    fn feature_end(&mut self, _idx: u64) -> Result<()> {
        if !self.has_written_first_record {
            self.has_written_first_record = true;
            self.csv.write_record(self.headers.clone())?;
        }

        let geom = &self.wkt_writer.out;
        self.csv.write_field(geom)?;
        self.wkt_writer.out.clear();

        for field in &self.current_row_props {
            self.csv.write_field(field)?;
        }
        self.csv.write_record(None::<&[u8]>)?;
        self.current_row_props.clear();

        Ok(())
    }
    fn properties_begin(&mut self) -> Result<()> {
        debug_assert!(self.current_row_props.is_empty());
        Ok(())
    }
    fn properties_end(&mut self) -> Result<()> {
        Ok(())
    }
    fn geometry_begin(&mut self) -> Result<()> {
        debug_assert!(!self.has_started_writing_geometry_in_this_row());
        Ok(())
    }
    fn geometry_end(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<W: Write> PropertyProcessor for CsvWriter<W> {
    fn property(&mut self, i: usize, colname: &str, colval: &ColumnValue) -> Result<bool> {
        // TODO: support mis-ordered properties?
        if self.has_written_first_record {
            assert_eq!(
                colname,
                &self.headers[i + 1],
                "CSV features must all have the same column names"
            );
        } else {
            self.headers.push(colname.to_string());
        }

        // TODO: support non-string colval
        self.current_row_props.push(colval.to_string());
        Ok(false)
    }
}

impl<W: Write> GeomProcessor for CsvWriter<W> {
    fn dimensions(&self) -> CoordDimensions {
        self.wkt_writer.dimensions()
    }
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        self.wkt_writer.xy(x, y, idx)
    }

    fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        m: Option<f64>,
        t: Option<f64>,
        tm: Option<u64>,
        idx: usize,
    ) -> Result<()> {
        self.wkt_writer.coordinate(x, y, z, m, t, tm, idx)
    }

    fn empty_point(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer.empty_point(self.offset_geom_idx(idx))
    }
    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer.point_begin(self.offset_geom_idx(idx))
    }
    fn point_end(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer.point_end(self.offset_geom_idx(idx))
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .multipoint_begin(size, self.offset_geom_idx(idx))
    }
    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer.multipoint_end(self.offset_geom_idx(idx))
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .linestring_begin(tagged, size, self.offset_geom_idx(idx))
    }
    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.wkt_writer
            .linestring_end(tagged, self.offset_geom_idx(idx))
    }
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .multilinestring_begin(size, self.offset_geom_idx(idx))
    }
    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer
            .multilinestring_end(self.offset_geom_idx(idx))
    }
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .polygon_begin(tagged, size, self.offset_geom_idx(idx))
    }
    fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.wkt_writer
            .polygon_end(tagged, self.offset_geom_idx(idx))
    }
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .multipolygon_begin(size, self.offset_geom_idx(idx))
    }
    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer.multipolygon_end(self.offset_geom_idx(idx))
    }
    fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .geometrycollection_begin(size, self.offset_geom_idx(idx))
    }
    fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer
            .geometrycollection_end(self.offset_geom_idx(idx))
    }
    fn circularstring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .circularstring_begin(size, self.offset_geom_idx(idx))
    }
    fn circularstring_end(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer
            .circularstring_end(self.offset_geom_idx(idx))
    }
    fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .compoundcurve_begin(size, self.offset_geom_idx(idx))
    }
    fn compoundcurve_end(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer.compoundcurve_end(self.offset_geom_idx(idx))
    }
    fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .curvepolygon_begin(size, self.offset_geom_idx(idx))
    }
    fn curvepolygon_end(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer.curvepolygon_end(self.offset_geom_idx(idx))
    }
    fn multicurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .multicurve_begin(size, self.offset_geom_idx(idx))
    }
    fn multicurve_end(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer.multicurve_end(self.offset_geom_idx(idx))
    }
    fn multisurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .multisurface_begin(size, self.offset_geom_idx(idx))
    }
    fn multisurface_end(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer.multisurface_end(self.offset_geom_idx(idx))
    }
    fn triangle_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .triangle_begin(tagged, size, self.offset_geom_idx(idx))
    }
    fn triangle_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.wkt_writer
            .triangle_end(tagged, self.offset_geom_idx(idx))
    }
    fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer
            .polyhedralsurface_begin(size, self.offset_geom_idx(idx))
    }
    fn polyhedralsurface_end(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer
            .polyhedralsurface_end(self.offset_geom_idx(idx))
    }
    fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.wkt_writer.tin_begin(size, self.offset_geom_idx(idx))
    }
    fn tin_end(&mut self, idx: usize) -> Result<()> {
        self.wkt_writer.tin_end(self.offset_geom_idx(idx))
    }
}

#[cfg(test)]
mod tests {
    use crate::ProcessToCsv;
    use serde_json::json;

    #[test]
    fn geojson_to_csv() {
        let input_geojson = json!({
            "type": "FeatureCollection",
            "name": "",
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

        let expected_output = r#"geometry,address,datetime,incident number,type
POINT(-122.329051 47.6069),904 7th Av,05/22/2019 12:55:00 PM,F190051945,Car Fire
POINT(-122.266529 47.515984),9610 53rd Av S,05/22/2019 12:55:00 PM,F190051946,Aid Response
"#;

        let actual_output = crate::geojson::GeoJson(&input_geojson.to_string())
            .to_csv()
            .unwrap();

        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn geojson_geom_collection_to_csv() {
        let input_geojson = json!({
            "type": "FeatureCollection",
            "name": "",
            "features": [
               {
                   "type": "Feature",
                   "geometry": {
                       "type": "Point",
                       "coordinates": [1.0, 45.0]
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
                       "type": "GeometryCollection",
                        "geometries": [
                           {
                               "type": "Point",
                               "coordinates": [2.1, 45.0]
                           },
                           {
                               "type": "Point",
                               "coordinates": [2.2, 45.0]
                           },
                       ]
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

        let expected_output = r#"geometry,address,datetime,incident number,type
POINT(1 45),904 7th Av,05/22/2019 12:55:00 PM,F190051945,Car Fire
"GEOMETRYCOLLECTION(POINT(2.1 45),POINT(2.2 45))",9610 53rd Av S,05/22/2019 12:55:00 PM,F190051946,Aid Response
"#;

        let actual_output = crate::geojson::GeoJson(&input_geojson.to_string())
            .to_csv()
            .unwrap();

        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn waterways() {
        let input_geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                { "type": "Feature", "properties": { "NAME": "Foo" }, "geometry": { "type": "Polygon", "coordinates": [ [ [ 1, 1], [2, 2], [1, 1] ] ] } },
                { "type": "Feature", "properties": { "NAME": "Bar" }, "geometry": { "type": "Polygon", "coordinates": [ [ [ 3, 1], [3, 2], [3, 1] ] ] } }
            ]
        }"#;

        let expected_output = r#"geometry,NAME
"POLYGON((1 1,2 2,1 1))",Foo
"POLYGON((3 1,3 2,3 1))",Bar
"#;

        let actual_output = crate::geojson::GeoJson(input_geojson).to_csv().unwrap();

        assert_eq!(expected_output, actual_output);
    }
}
