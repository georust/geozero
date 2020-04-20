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
