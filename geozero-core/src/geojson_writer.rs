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
    fn dataset_begin(&mut self, name: Option<&str>) -> Result<()> {
        let _ = self.out.write(
            br#"{
"type": "FeatureCollection",
"name": ""#,
        )?;
        if let Some(name) = name {
            let _ = self.out.write(name.as_bytes())?;
        }
        let _ = self.out.write(
            br#"",
"features": ["#,
        )?;
        Ok(())
    }
    fn dataset_end(&mut self) -> Result<()> {
        let _ = self.out.write(b"]}")?;
        Ok(())
    }
    fn feature_begin(&mut self, idx: u64) -> Result<()> {
        if idx > 0 {
            let _ = self.out.write(b",\n")?;
        }
        let _ = self.out.write(br#"{"type": "Feature", "#)?;
        Ok(())
    }
    fn feature_end(&mut self, _idx: u64) -> Result<()> {
        let _ = self.out.write(b"}")?;
        Ok(())
    }
    fn properties_begin(&mut self) -> Result<()> {
        let _ = self.out.write(br#""properties": {"#)?;
        Ok(())
    }
    fn properties_end(&mut self) -> Result<()> {
        let _ = self.out.write(b"}, ")?; //TODO: support also properties after geometry!
        Ok(())
    }
    fn geometry_begin(&mut self) -> Result<()> {
        let _ = self.out.write(br#""geometry": "#)?;
        Ok(())
    }
    fn geometry_end(&mut self) -> Result<()> {
        Ok(())
    }
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

fn write_num_prop<'a, W: Write>(out: &'a mut W, colname: &str, v: &dyn Display) -> Result<()> {
    let _ = out.write(&format!(r#""{}": {}"#, colname, v).as_bytes())?;
    Ok(())
}

fn write_str_prop<'a, W: Write>(out: &'a mut W, colname: &str, v: &dyn Display) -> Result<()> {
    let _ = out.write(&format!(r#""{}": "{}""#, colname, v).as_bytes())?;
    Ok(())
}

impl<W: Write> PropertyProcessor for GeoJsonWriter<'_, W> {
    fn property(&mut self, i: usize, colname: &str, colval: &ColumnValue) -> Result<bool> {
        if i > 0 {
            let _ = self.out.write(b", ")?;
        }
        match colval {
            ColumnValue::Byte(v) => write_num_prop(self.out, colname, &v)?,
            ColumnValue::UByte(v) => write_num_prop(self.out, colname, &v)?,
            ColumnValue::Bool(v) => write_num_prop(self.out, colname, &v)?,
            ColumnValue::Short(v) => write_num_prop(self.out, colname, &v)?,
            ColumnValue::UShort(v) => write_num_prop(self.out, colname, &v)?,
            ColumnValue::Int(v) => write_num_prop(self.out, colname, &v)?,
            ColumnValue::UInt(v) => write_num_prop(self.out, colname, &v)?,
            ColumnValue::Long(v) => write_num_prop(self.out, colname, &v)?,
            ColumnValue::ULong(v) => write_num_prop(self.out, colname, &v)?,
            ColumnValue::Float(v) => write_num_prop(self.out, colname, &v)?,
            ColumnValue::Double(v) => write_num_prop(self.out, colname, &v)?,
            ColumnValue::String(v) => write_str_prop(self.out, colname, &v)?,
            ColumnValue::Json(_v) => (),
            ColumnValue::DateTime(v) => write_str_prop(self.out, colname, &v)?,
            ColumnValue::Binary(_v) => (),
        };
        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::geojson_reader::read_geojson;

    #[test]
    fn line_string() -> Result<()> {
        let geojson = r#"{"type": "LineString", "coordinates": [[1875038.447610231,-3269648.6879248763],[1874359.641504197,-3270196.812984864],[1874141.0428635243,-3270953.7840121365],[1874440.1778162003,-3271619.4315206874],[1876396.0598222911,-3274138.747656357],[1876442.0805243007,-3275052.60551469],[1874739.312657555,-3275457.333765534]]}"#;
        let mut out: Vec<u8> = Vec::new();
        assert!(read_geojson(geojson.as_bytes(), &mut GeoJsonWriter::new(&mut out)).is_ok());
        let jsonout = std::str::from_utf8(&out).unwrap();
        assert_eq!(jsonout, geojson);
        Ok(())
    }
}
