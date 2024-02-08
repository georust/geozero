use std::io::Write;

use crate::error::Result;
use crate::geojson::GeoJsonWriter;
use crate::{FeatureProcessor, GeomProcessor, PropertyProcessor};

/// Line Delimited GeoJSON Writer: One feature per line.
///
/// See <https://jsonlines.org>
pub struct GeoJsonLineWriter<W: Write> {
    /// We use a count of the number of contexts entered to decide when to add a newline character
    /// and finish a line. The [newline-delimited GeoJSON
    /// spec](https://datatracker.ietf.org/doc/html/rfc8142) defines that any type of GeoJSON
    /// objects can be written as an object on a single line. Therefore, we can't solely add
    /// newlines in `feature_end`. If the object on this line is a Point geometry, then we need to
    /// add a newline character in `point_end`, because `feature_end` will never be called.
    ///
    /// Note that this approach is not resilient to malformed input. If the number of begin and end
    /// calls do not match, newline characters will not be correctly added.
    open_contexts: usize,
    line_writer: GeoJsonWriter<W>,
}

impl<W: Write> GeoJsonLineWriter<W> {
    pub fn new(out: W) -> Self {
        Self {
            open_contexts: 0,
            line_writer: GeoJsonWriter::new(out),
        }
    }

    fn write_newline(&mut self) -> Result<()> {
        self.line_writer.out.write_all(b"\n")?;
        Ok(())
    }

    fn begin_context(&mut self) {
        self.open_contexts += 1;
    }

    fn end_context(&mut self) -> Result<()> {
        self.open_contexts -= 1;
        if self.open_contexts == 0 {
            self.write_newline()?;
        }
        Ok(())
    }
}

impl<W: Write> FeatureProcessor for GeoJsonLineWriter<W> {
    fn feature_begin(&mut self, _idx: u64) -> Result<()> {
        self.begin_context();
        self.line_writer.feature_begin(0)?;
        Ok(())
    }

    fn feature_end(&mut self, _idx: u64) -> Result<()> {
        self.line_writer.feature_end(0)?;
        self.end_context()?;
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
        self.begin_context();
        self.line_writer.empty_point(idx)?;
        self.end_context()
    }

    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.begin_context();
        self.line_writer.point_begin(idx)
    }

    fn point_end(&mut self, idx: usize) -> Result<()> {
        self.line_writer.point_end(idx)?;
        self.end_context()
    }

    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.begin_context();
        self.line_writer.multipoint_begin(size, idx)
    }

    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.line_writer.multipoint_end(idx)?;
        self.end_context()
    }

    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.begin_context();
        self.line_writer.linestring_begin(tagged, size, idx)
    }

    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.line_writer.linestring_end(tagged, idx)?;
        self.end_context()
    }

    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.begin_context();
        self.line_writer.multilinestring_begin(size, idx)
    }

    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        self.line_writer.multilinestring_end(idx)?;
        self.end_context()
    }

    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.begin_context();
        self.line_writer.polygon_begin(tagged, size, idx)
    }

    fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.line_writer.polygon_end(tagged, idx)?;
        self.end_context()
    }

    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.begin_context();
        self.line_writer.multipolygon_begin(size, idx)
    }

    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        self.line_writer.multipolygon_end(idx)?;
        self.end_context()
    }

    fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.begin_context();
        self.line_writer.geometrycollection_begin(size, idx)
    }

    fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        self.line_writer.geometrycollection_end(idx)?;
        self.end_context()
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
        assert_json_lines_eq(&out, input);
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
        a.lines().zip(b.lines()).for_each(|(a_line, b_line)| {
            println!("a: {}", a_line);
            println!("b: {}", b_line);

            let a_val: serde_json::Value = serde_json::from_str(a_line).unwrap();
            let b_val: serde_json::Value = serde_json::from_str(b_line).unwrap();
            assert_eq!(a_val, b_val);
        })
    }
}
