use geozero::error::Result;
use geozero::{ColumnValue, FeatureProcessor, GeomProcessor, PropertyProcessor};
use std::fmt::Display;
use std::io::Write;

pub struct GeoJsonWriter<'a, W: Write> {
    out: &'a mut W,
}

impl<'a, W: Write> GeoJsonWriter<'a, W> {
    pub fn new(out: &'a mut W) -> GeoJsonWriter<'a, W> {
        GeoJsonWriter { out }
    }
    fn comma(&mut self, idx: usize) -> Result<()> {
        if idx > 0 {
            let _ = self.out.write(b",")?;
        }
        Ok(())
    }
}

impl<W: Write> FeatureProcessor for GeoJsonWriter<'_, W> {
    fn dataset_begin(&mut self, name: Option<&str>) {
        self.out
            .write(
                br#"{
"type": "FeatureCollection",
"name": ""#,
            )
            .unwrap();
        if let Some(name) = name {
            self.out.write(name.as_bytes()).unwrap();
        }
        self.out
            .write(
                br#"",
"features": ["#,
            )
            .unwrap();
    }
    fn dataset_end(&mut self) {
        self.out.write(b"]}").unwrap();
    }
    fn feature_begin(&mut self, idx: u64) {
        if idx > 0 {
            self.out.write(b",\n").unwrap();
        }
        self.out.write(br#"{"type": "Feature", "#).unwrap();
    }
    fn feature_end(&mut self, _idx: u64) {
        self.out.write(b"}").unwrap();
    }
    fn properties_begin(&mut self) {
        self.out.write(br#""properties": {"#).unwrap();
    }
    fn properties_end(&mut self) {
        self.out.write(b"}, ").unwrap(); //TODO: support also properties after geometry!
    }
    fn geometry_begin(&mut self) {
        self.out.write(br#""geometry": "#).unwrap();
    }
    fn geometry_end(&mut self) {}
}

impl<W: Write> GeomProcessor for GeoJsonWriter<'_, W> {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        self.comma(idx)?;
        let _ = self.out.write(&format!("[{},{}]", x, y).as_bytes())?;
        Ok(())
    }
    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.comma(idx)?;
        let _ = self.out.write(br#"{"type": "Point", "coordinates": "#)?;
        Ok(())
    }
    fn point_end(&mut self, _idx: usize) -> Result<()> {
        let _ = self.out.write(b"}")?;
        Ok(())
    }
    fn multipoint_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.comma(idx)?;
        let _ = self
            .out
            .write(br#"{"type": "MultiPoint", "coordinates": ["#)?;
        Ok(())
    }
    fn multipoint_end(&mut self, _idx: usize) -> Result<()> {
        let _ = self.out.write(b"]}")?;
        Ok(())
    }
    fn linestring_begin(&mut self, tagged: bool, _size: usize, idx: usize) -> Result<()> {
        self.comma(idx)?;
        if tagged {
            let _ = self
                .out
                .write(br#"{"type": "LineString", "coordinates": ["#)?;
        } else {
            let _ = self.out.write(b"[")?;
        }
        Ok(())
    }
    fn linestring_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        if tagged {
            let _ = self.out.write(b"]}")?;
        } else {
            let _ = self.out.write(b"]")?;
        }
        Ok(())
    }
    fn multilinestring_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.comma(idx)?;
        let _ = self
            .out
            .write(br#"{"type": "MultiLineString", "coordinates": ["#)?;
        Ok(())
    }
    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        let _ = self.out.write(b"]}")?;
        Ok(())
    }
    fn polygon_begin(&mut self, tagged: bool, _size: usize, idx: usize) -> Result<()> {
        self.comma(idx)?;
        if tagged {
            let _ = self.out.write(br#"{"type": "Polygon", "coordinates": ["#)?;
        } else {
            let _ = self.out.write(b"[")?;
        }
        Ok(())
    }
    fn polygon_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        if tagged {
            let _ = self.out.write(b"]}")?;
        } else {
            let _ = self.out.write(b"]")?;
        }
        Ok(())
    }
    fn multipolygon_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.comma(idx)?;
        let _ = self
            .out
            .write(br#"{"type": "MultiPolygon", "coordinates": ["#)?;
        Ok(())
    }
    fn multipolygon_end(&mut self, _idx: usize) -> Result<()> {
        let _ = self.out.write(b"]}")?;
        Ok(())
    }
}

fn write_num_prop<'a, W: Write>(out: &'a mut W, colname: &str, v: &dyn Display) -> usize {
    out.write(&format!(r#""{}": {}"#, colname, v).as_bytes())
        .unwrap()
}

fn write_str_prop<'a, W: Write>(out: &'a mut W, colname: &str, v: &dyn Display) -> usize {
    out.write(&format!(r#""{}": "{}""#, colname, v).as_bytes())
        .unwrap()
}

impl<W: Write> PropertyProcessor for GeoJsonWriter<'_, W> {
    fn property(&mut self, i: usize, colname: &str, colval: &ColumnValue) -> bool {
        if i > 0 {
            self.out.write(b", ").unwrap();
        }
        match colval {
            ColumnValue::Byte(v) => write_num_prop(self.out, colname, &v),
            ColumnValue::UByte(v) => write_num_prop(self.out, colname, &v),
            ColumnValue::Bool(v) => write_num_prop(self.out, colname, &v),
            ColumnValue::Short(v) => write_num_prop(self.out, colname, &v),
            ColumnValue::UShort(v) => write_num_prop(self.out, colname, &v),
            ColumnValue::Int(v) => write_num_prop(self.out, colname, &v),
            ColumnValue::UInt(v) => write_num_prop(self.out, colname, &v),
            ColumnValue::Long(v) => write_num_prop(self.out, colname, &v),
            ColumnValue::ULong(v) => write_num_prop(self.out, colname, &v),
            ColumnValue::Float(v) => write_num_prop(self.out, colname, &v),
            ColumnValue::Double(v) => write_num_prop(self.out, colname, &v),
            ColumnValue::String(v) => write_str_prop(self.out, colname, &v),
            ColumnValue::Json(_v) => 0,
            ColumnValue::DateTime(v) => write_str_prop(self.out, colname, &v),
            ColumnValue::Binary(_v) => 0,
        };
        false
    }
}
