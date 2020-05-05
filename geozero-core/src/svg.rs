use geozero::error::Result;
use geozero::{FeatureProcessor, GeomProcessor, PropertyProcessor};
use std::io::Write;

pub struct SvgWriter<'a, W: Write> {
    out: &'a mut W,
    invert_y: bool,
    xmin: f64,
    ymin: f64,
    xmax: f64,
    ymax: f64,
    width: u32,
    height: u32,
}

impl<'a, W: Write> SvgWriter<'a, W> {
    pub fn new(out: &'a mut W, invert_y: bool) -> SvgWriter<'a, W> {
        SvgWriter {
            out,
            invert_y,
            xmin: 0.0,
            ymin: 0.0,
            xmax: 0.0,
            ymax: 0.0,
            width: 0,
            height: 0,
        }
    }
    pub fn set_dimensions(
        &mut self,
        xmin: f64,
        ymin: f64,
        xmax: f64,
        ymax: f64,
        width: u32,
        height: u32,
    ) {
        self.xmin = xmin;
        self.xmax = xmax;
        if self.invert_y {
            self.ymin = -ymax;
            self.ymax = -ymin;
        } else {
            self.ymin = ymin;
            self.ymax = ymax;
        }
        self.width = width;
        self.height = height;
    }
}

impl<W: Write> FeatureProcessor for SvgWriter<'_, W> {
    fn dataset_begin(&mut self, name: Option<&str>) -> Result<()> {
        let _ = self.out.write(
            br#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" version="1.2" baseProfile="tiny" "#,
        )?;
        let _ = self
            .out
            .write(&format!("width=\"{}\" height=\"{}\" ", self.width, self.height).as_bytes())?;
        let _ = self.out.write(
            &format!(
                "viewBox=\"{} {} {} {}\" ",
                self.xmin,
                self.ymin,
                self.xmax - self.xmin,
                self.ymax - self.ymin
            )
            .as_bytes(),
        )?;
        let _ = self.out.write(
            br#"stroke-linecap="round" stroke-linejoin="round">
<g id=""#,
        )?;
        if let Some(name) = name {
            let _ = self.out.write(name.as_bytes())?;
        }
        let _ = self.out.write(br#"">"#)?;
        Ok(())
    }
    fn dataset_end(&mut self) -> Result<()> {
        let _ = self.out.write(b"\n</g>\n</svg>")?;
        Ok(())
    }
    fn feature_begin(&mut self, _idx: u64) -> Result<()> {
        let _ = self.out.write(b"\n")?;
        Ok(())
    }
    fn feature_end(&mut self, _idx: u64) -> Result<()> {
        Ok(())
    }
}

impl<W: Write> GeomProcessor for SvgWriter<'_, W> {
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
        let y = if self.invert_y { -y } else { y };
        let _ = self.out.write(&format!("{} {} ", x, y).as_bytes())?;
        Ok(())
    }
    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        let _ = self.out.write(br#"<path d="M "#)?;
        Ok(())
    }
    fn point_end(&mut self, _idx: usize) -> Result<()> {
        let _ = self.out.write(br#"Z"/>"#)?;
        Ok(())
    }
    fn linestring_begin(&mut self, tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        if tagged {
            let _ = self.out.write(br#"<path d=""#)?;
        } else {
            let _ = self.out.write(b"M ")?;
        }
        Ok(())
    }
    fn linestring_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        if tagged {
            let _ = self.out.write(br#""/>"#)?;
        } else {
            let _ = self.out.write(b"Z ")?;
        }
        Ok(())
    }
    fn multilinestring_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        let _ = self.out.write(br#"<path d=""#)?;
        Ok(())
    }
    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        let _ = self.out.write(br#""/>"#)?;
        Ok(())
    }
    fn polygon_begin(&mut self, _tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        let _ = self.out.write(br#"<path d=""#)?;
        Ok(())
    }
    fn polygon_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        let _ = self.out.write(br#""/>"#)?;
        Ok(())
    }
}

impl<W: Write> PropertyProcessor for SvgWriter<'_, W> {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::geojson_reader::read_geojson;

    #[test]
    fn line_string() -> Result<()> {
        let geojson = r#"{"type": "LineString", "coordinates": [[1875038.447610231,-3269648.6879248763],[1874359.641504197,-3270196.812984864],[1874141.0428635243,-3270953.7840121365],[1874440.1778162003,-3271619.4315206874],[1876396.0598222911,-3274138.747656357],[1876442.0805243007,-3275052.60551469],[1874739.312657555,-3275457.333765534]]}"#;
        let mut svg_data: Vec<u8> = Vec::new();
        let mut svg = SvgWriter::new(&mut svg_data, false);
        assert!(read_geojson(geojson.as_bytes(), &mut svg).is_ok());
        assert_eq!(
            std::str::from_utf8(&svg_data).unwrap(),
            r#"<path d="1875038.447610231 -3269648.6879248763 1874359.641504197 -3270196.812984864 1874141.0428635243 -3270953.7840121365 1874440.1778162003 -3271619.4315206874 1876396.0598222911 -3274138.747656357 1876442.0805243007 -3275052.60551469 1874739.312657555 -3275457.333765534 "/>"#
        );
        Ok(())
    }

    #[test]
    fn feature() -> Result<()> {
        let geojson = r#"{"type": "Feature", "properties": {"id": "NZL", "name": "New Zealand"}, "geometry": {"type": "MultiPolygon", "coordinates": [[[[173.020375,-40.919052],[173.247234,-41.331999],[173.958405,-40.926701],[174.247587,-41.349155],[174.248517,-41.770008],[173.876447,-42.233184],[173.22274,-42.970038],[172.711246,-43.372288],[173.080113,-43.853344],[172.308584,-43.865694],[171.452925,-44.242519],[171.185138,-44.897104],[170.616697,-45.908929],[169.831422,-46.355775],[169.332331,-46.641235],[168.411354,-46.619945],[167.763745,-46.290197],[166.676886,-46.219917],[166.509144,-45.852705],[167.046424,-45.110941],[168.303763,-44.123973],[168.949409,-43.935819],[169.667815,-43.555326],[170.52492,-43.031688],[171.12509,-42.512754],[171.569714,-41.767424],[171.948709,-41.514417],[172.097227,-40.956104],[172.79858,-40.493962],[173.020375,-40.919052]]],[[[174.612009,-36.156397],[175.336616,-37.209098],[175.357596,-36.526194],[175.808887,-36.798942],[175.95849,-37.555382],[176.763195,-37.881253],[177.438813,-37.961248],[178.010354,-37.579825],[178.517094,-37.695373],[178.274731,-38.582813],[177.97046,-39.166343],[177.206993,-39.145776],[176.939981,-39.449736],[177.032946,-39.879943],[176.885824,-40.065978],[176.508017,-40.604808],[176.01244,-41.289624],[175.239567,-41.688308],[175.067898,-41.425895],[174.650973,-41.281821],[175.22763,-40.459236],[174.900157,-39.908933],[173.824047,-39.508854],[173.852262,-39.146602],[174.574802,-38.797683],[174.743474,-38.027808],[174.697017,-37.381129],[174.292028,-36.711092],[174.319004,-36.534824],[173.840997,-36.121981],[173.054171,-35.237125],[172.636005,-34.529107],[173.007042,-34.450662],[173.551298,-35.006183],[174.32939,-35.265496],[174.612009,-36.156397]]]]}}"#;
        let mut svg_data: Vec<u8> = Vec::new();
        let mut svg = SvgWriter::new(&mut svg_data, false);
        // svg.set_dimensions(bbox.get(0), bbox.get(1), bbox.get(2), bbox.get(3), 800, 400);
        assert!(read_geojson(geojson.as_bytes(), &mut svg).is_ok());
        assert_eq!(
            std::str::from_utf8(&svg_data).unwrap(),
            r#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" version="1.2" baseProfile="tiny" width="0" height="0" viewBox="0 0 0 0" stroke-linecap="round" stroke-linejoin="round">
<g id="">
<path d="M 173.020375 -40.919052 173.247234 -41.331999 173.958405 -40.926701 174.247587 -41.349155 174.248517 -41.770008 173.876447 -42.233184 173.22274 -42.970038 172.711246 -43.372288 173.080113 -43.853344 172.308584 -43.865694 171.452925 -44.242519 171.185138 -44.897104 170.616697 -45.908929 169.831422 -46.355775 169.332331 -46.641235 168.411354 -46.619945 167.763745 -46.290197 166.676886 -46.219917 166.509144 -45.852705 167.046424 -45.110941 168.303763 -44.123973 168.949409 -43.935819 169.667815 -43.555326 170.52492 -43.031688 171.12509 -42.512754 171.569714 -41.767424 171.948709 -41.514417 172.097227 -40.956104 172.79858 -40.493962 173.020375 -40.919052 Z "/><path d="M 174.612009 -36.156397 175.336616 -37.209098 175.357596 -36.526194 175.808887 -36.798942 175.95849 -37.555382 176.763195 -37.881253 177.438813 -37.961248 178.010354 -37.579825 178.517094 -37.695373 178.274731 -38.582813 177.97046 -39.166343 177.206993 -39.145776 176.939981 -39.449736 177.032946 -39.879943 176.885824 -40.065978 176.508017 -40.604808 176.01244 -41.289624 175.239567 -41.688308 175.067898 -41.425895 174.650973 -41.281821 175.22763 -40.459236 174.900157 -39.908933 173.824047 -39.508854 173.852262 -39.146602 174.574802 -38.797683 174.743474 -38.027808 174.697017 -37.381129 174.292028 -36.711092 174.319004 -36.534824 173.840997 -36.121981 173.054171 -35.237125 172.636005 -34.529107 173.007042 -34.450662 173.551298 -35.006183 174.32939 -35.265496 174.612009 -36.156397 Z "/>
</g>
</svg>"#
        );
        Ok(())
    }
}
