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
    fn dataset_begin(&mut self, name: Option<&str>) {
        self.out
            .write(
                br#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" version="1.2" baseProfile="tiny" "#,
            )
            .unwrap();
        self.out
            .write(&format!("width=\"{}\" height=\"{}\" ", self.width, self.height).as_bytes())
            .unwrap();
        self.out
            .write(
                &format!(
                    "viewBox=\"{} {} {} {}\" ",
                    self.xmin,
                    self.ymin,
                    self.xmax - self.xmin,
                    self.ymax - self.ymin
                )
                .as_bytes(),
            )
            .unwrap();
        self.out
            .write(
                br#"stroke-linecap="round" stroke-linejoin="round">
<g id=""#,
            )
            .unwrap();
        if let Some(name) = name {
            self.out.write(name.as_bytes()).unwrap();
        }
        self.out.write(br#"">"#).unwrap();
    }
    fn dataset_end(&mut self) {
        self.out.write(b"\n</g>\n</svg>").unwrap();
    }
    fn feature_begin(&mut self, _idx: u64) {
        self.out.write(b"\n").unwrap();
    }
    fn feature_end(&mut self, _idx: u64) {}
}

impl<W: Write> GeomProcessor for SvgWriter<'_, W> {
    fn xy(&mut self, x: f64, y: f64, _idx: usize) {
        let y = if self.invert_y { -y } else { y };
        self.out.write(&format!("{} {} ", x, y).as_bytes()).unwrap();
    }
    fn point_begin(&mut self, _idx: usize) {
        self.out.write(br#"<path d="M "#).unwrap();
    }
    fn point_end(&mut self, _idx: usize) {
        self.out.write(br#"Z"/>"#).unwrap();
    }
    fn linestring_begin(&mut self, tagged: bool, _size: usize, _idx: usize) {
        if tagged {
            self.out.write(br#"<path d=""#).unwrap();
        } else {
            self.out.write(b"M ").unwrap();
        }
    }
    fn linestring_end(&mut self, tagged: bool, _idx: usize) {
        if tagged {
            self.out.write(br#""/>"#).unwrap();
        } else {
            self.out.write(b"Z ").unwrap();
        }
    }
    fn multilinestring_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(br#"<path d=""#).unwrap();
    }
    fn multilinestring_end(&mut self, _idx: usize) {
        self.out.write(br#""/>"#).unwrap();
    }
    fn polygon_begin(&mut self, _tagged: bool, _size: usize, _idx: usize) {
        self.out.write(br#"<path d=""#).unwrap();
    }
    fn polygon_end(&mut self, _tagged: bool, _idx: usize) {
        self.out.write(br#""/>"#).unwrap();
    }
}

impl<W: Write> PropertyProcessor for SvgWriter<'_, W> {}
