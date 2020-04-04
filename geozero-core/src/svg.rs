use geozero_api::GeomReader;
use std::io::Write;

pub struct SvgEmitter<'a, W: Write> {
    out: &'a mut W,
    invert_y: bool,
}

impl<'a, W: Write> SvgEmitter<'a, W> {
    pub fn new(out: &'a mut W, invert_y: bool) -> SvgEmitter<'a, W> {
        SvgEmitter { out, invert_y }
    }
}

impl<W: Write> GeomReader for SvgEmitter<'_, W> {
    fn pointxy(&mut self, x: f64, y: f64, _idx: usize) {
        let y = if self.invert_y { -y } else { y };
        self.out.write(&format!("{} {} ", x, y).as_bytes()).unwrap();
    }
    fn point_begin(&mut self, _idx: usize) {
        self.out.write(br#"<path d="M "#).unwrap();
    }
    fn point_end(&mut self) {
        self.out.write(br#"Z"/>"#).unwrap();
    }
    fn line_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(br#"<path d=""#).unwrap();
    }
    fn line_end(&mut self, _idx: usize) {
        self.out.write(br#""/>"#).unwrap();
    }
    fn multiline_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(br#"<path d=""#).unwrap();
    }
    fn multiline_end(&mut self) {
        self.out.write(br#""/>"#).unwrap();
    }
    fn ring_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(b"M ").unwrap();
    }
    fn ring_end(&mut self, _idx: usize) {
        self.out.write(b"Z ").unwrap();
    }
    fn poly_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(br#"<path d=""#).unwrap();
    }
    fn poly_end(&mut self, _idx: usize) {
        self.out.write(br#""/>"#).unwrap();
    }
    fn subpoly_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(br#"<path d=""#).unwrap();
    }
    fn subpoly_end(&mut self, _idx: usize) {
        self.out.write(br#""/>"#).unwrap();
    }
}
