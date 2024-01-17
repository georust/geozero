use std::io::Write;

use crate::error::Result;
use crate::geojson::GeoJsonWriter;
use crate::{FeatureProcessor, GeomProcessor, PropertyProcessor};

/// Line Delimited GeoJSON Writer: One feature per line.
///
/// See <https://jsonlines.org>
pub struct GeoJsonLineWriter<W: Write> {
    line_writer: GeoJsonWriter<W>,
}

impl<W: Write> GeoJsonLineWriter<W> {
    pub fn new(out: W) -> Self {
        Self {
            line_writer: GeoJsonWriter::new(out),
        }
    }
}

impl<W: Write> FeatureProcessor for GeoJsonLineWriter<W> {
    fn dataset_begin(&mut self, _name: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn dataset_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn feature_begin(&mut self, _idx: u64) -> Result<()> {
        self.line_writer.feature_begin(0)
    }

    fn feature_end(&mut self, _idx: u64) -> Result<()> {
        self.line_writer.feature_end(0)?;
        self.line_writer.out.write_all(b"\n")?;
        Ok(())
    }

    fn properties_begin(&mut self) -> Result<()> {
        self.line_writer.properties_begin()
    }

    fn properties_end(&mut self) -> Result<()> {
        self.line_writer.properties_end()
    }

    fn geometry_begin(&mut self) -> Result<()> {
        self.line_writer.geometry_begin()
    }

    fn geometry_end(&mut self) -> Result<()> {
        self.line_writer.geometry_end()
    }
}

impl<W: Write> GeomProcessor for GeoJsonLineWriter<W> {
    fn dimensions(&self) -> crate::CoordDimensions {
        self.line_writer.dimensions()
    }

    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        self.line_writer.xy(x, y, idx)
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
        self.line_writer.coordinate(x, y, z, m, t, tm, idx)
    }

    fn empty_point(&mut self, idx: usize) -> Result<()> {
        self.line_writer.empty_point(idx)
    }

    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.line_writer.point_begin(idx)
    }

    fn point_end(&mut self, idx: usize) -> Result<()> {
        self.line_writer.point_end(idx)
    }

    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.line_writer.multipoint_begin(size, idx)
    }

    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.line_writer.multipoint_end(idx)
    }

    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.line_writer.linestring_begin(tagged, size, idx)
    }

    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.line_writer.linestring_end(tagged, idx)
    }

    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.line_writer.multilinestring_begin(size, idx)
    }

    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        self.line_writer.multilinestring_end(idx)
    }

    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.line_writer.polygon_begin(tagged, size, idx)
    }

    fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.line_writer.polygon_end(tagged, idx)
    }

    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.line_writer.multipolygon_begin(size, idx)
    }

    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        self.line_writer.multipolygon_end(idx)
    }

    fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.line_writer.geometrycollection_begin(size, idx)
    }

    fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        self.line_writer.geometrycollection_end(idx)
    }
}

impl<W: Write> PropertyProcessor for GeoJsonLineWriter<W> {
    fn property(&mut self, idx: usize, name: &str, value: &crate::ColumnValue) -> Result<bool> {
        self.line_writer.property(idx, name, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geojson::read_geojson_lines;

    #[ignore = "fails because no newlines are written. feature_end is never called."]
    #[test]
    fn good_geometries() {
        let input = r#"{ "type": "Point", "coordinates": [1.1, 1.2] }
    { "type": "Point", "coordinates": [2.1, 2.2] }
    { "type": "Point", "coordinates": [3.1, 3.2] }
    "#;
        let mut out: Vec<u8> = Vec::new();
        assert!(
            read_geojson_lines(input.as_bytes(), &mut GeoJsonLineWriter::new(&mut out)).is_ok()
        );
        assert_json_eq(&out, input);
    }

    #[test]
    fn good_features() {
        let input = r#"{ "type": "Feature", "geometry": { "type": "Point", "coordinates": [1.1, 1.2] }, "properties": { "name": "first" } }
{ "type": "Feature", "geometry": { "type": "Point", "coordinates": [2.1, 2.2] }, "properties": { "name": "second" } }
{ "type": "Feature", "geometry": { "type": "Point", "coordinates": [3.1, 3.3] }, "properties": { "name": "third" } }
"#;
        let mut out: Vec<u8> = Vec::new();
        assert!(
            read_geojson_lines(input.as_bytes(), &mut GeoJsonLineWriter::new(&mut out)).is_ok()
        );
        assert_json_lines_eq(&out, input);
    }

    fn assert_json_lines_eq(a: &[u8], b: &str) {
        let a = std::str::from_utf8(a).unwrap();
        dbg!(a);
        a.lines().zip(b.lines()).for_each(|(a_line, b_line)| {
            dbg!(a);

            let a_val: serde_json::Value = serde_json::from_str(a_line).unwrap();
            let b_val: serde_json::Value = serde_json::from_str(b_line).unwrap();
            assert_eq!(a_val, b_val);
        })
    }
}
