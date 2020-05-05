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

    #[test]
    fn feature_collection() -> Result<()> {
        let geojson = r#"{"type": "FeatureCollection", "name": "countries", "features": [{"type": "Feature", "properties": {"id": "NZL", "name": "New Zealand"}, "geometry": {"type": "MultiPolygon", "coordinates": [[[[173.020375,-40.919052],[173.247234,-41.331999],[173.958405,-40.926701],[174.247587,-41.349155],[174.248517,-41.770008],[173.876447,-42.233184],[173.22274,-42.970038],[172.711246,-43.372288],[173.080113,-43.853344],[172.308584,-43.865694],[171.452925,-44.242519],[171.185138,-44.897104],[170.616697,-45.908929],[169.831422,-46.355775],[169.332331,-46.641235],[168.411354,-46.619945],[167.763745,-46.290197],[166.676886,-46.219917],[166.509144,-45.852705],[167.046424,-45.110941],[168.303763,-44.123973],[168.949409,-43.935819],[169.667815,-43.555326],[170.52492,-43.031688],[171.12509,-42.512754],[171.569714,-41.767424],[171.948709,-41.514417],[172.097227,-40.956104],[172.79858,-40.493962],[173.020375,-40.919052]]],[[[174.612009,-36.156397],[175.336616,-37.209098],[175.357596,-36.526194],[175.808887,-36.798942],[175.95849,-37.555382],[176.763195,-37.881253],[177.438813,-37.961248],[178.010354,-37.579825],[178.517094,-37.695373],[178.274731,-38.582813],[177.97046,-39.166343],[177.206993,-39.145776],[176.939981,-39.449736],[177.032946,-39.879943],[176.885824,-40.065978],[176.508017,-40.604808],[176.01244,-41.289624],[175.239567,-41.688308],[175.067898,-41.425895],[174.650973,-41.281821],[175.22763,-40.459236],[174.900157,-39.908933],[173.824047,-39.508854],[173.852262,-39.146602],[174.574802,-38.797683],[174.743474,-38.027808],[174.697017,-37.381129],[174.292028,-36.711092],[174.319004,-36.534824],[173.840997,-36.121981],[173.054171,-35.237125],[172.636005,-34.529107],[173.007042,-34.450662],[173.551298,-35.006183],[174.32939,-35.265496],[174.612009,-36.156397]]]]}}]}"#;
        let mut out: Vec<u8> = Vec::new();
        assert!(read_geojson(geojson.as_bytes(), &mut GeoJsonWriter::new(&mut out)).is_ok());
        let jsonout = std::str::from_utf8(&out).unwrap();
        // Properties missing!
        assert_eq!(
            jsonout,
            r#"{
"type": "FeatureCollection",
"name": "",
"features": [{"type": "Feature", "properties": {}, "geometry": {"type": "MultiPolygon", "coordinates": [[[[173.020375,-40.919052],[173.247234,-41.331999],[173.958405,-40.926701],[174.247587,-41.349155],[174.248517,-41.770008],[173.876447,-42.233184],[173.22274,-42.970038],[172.711246,-43.372288],[173.080113,-43.853344],[172.308584,-43.865694],[171.452925,-44.242519],[171.185138,-44.897104],[170.616697,-45.908929],[169.831422,-46.355775],[169.332331,-46.641235],[168.411354,-46.619945],[167.763745,-46.290197],[166.676886,-46.219917],[166.509144,-45.852705],[167.046424,-45.110941],[168.303763,-44.123973],[168.949409,-43.935819],[169.667815,-43.555326],[170.52492,-43.031688],[171.12509,-42.512754],[171.569714,-41.767424],[171.948709,-41.514417],[172.097227,-40.956104],[172.79858,-40.493962],[173.020375,-40.919052]]],[[[174.612009,-36.156397],[175.336616,-37.209098],[175.357596,-36.526194],[175.808887,-36.798942],[175.95849,-37.555382],[176.763195,-37.881253],[177.438813,-37.961248],[178.010354,-37.579825],[178.517094,-37.695373],[178.274731,-38.582813],[177.97046,-39.166343],[177.206993,-39.145776],[176.939981,-39.449736],[177.032946,-39.879943],[176.885824,-40.065978],[176.508017,-40.604808],[176.01244,-41.289624],[175.239567,-41.688308],[175.067898,-41.425895],[174.650973,-41.281821],[175.22763,-40.459236],[174.900157,-39.908933],[173.824047,-39.508854],[173.852262,-39.146602],[174.574802,-38.797683],[174.743474,-38.027808],[174.697017,-37.381129],[174.292028,-36.711092],[174.319004,-36.534824],[173.840997,-36.121981],[173.054171,-35.237125],[172.636005,-34.529107],[173.007042,-34.450662],[173.551298,-35.006183],[174.32939,-35.265496],[174.612009,-36.156397]]]]}}]}"#
        );
        Ok(())
    }
}
