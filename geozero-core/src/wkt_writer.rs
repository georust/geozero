use geozero::error::Result;
use geozero::{CoordDimensions, FeatureProcessor, GeomProcessor, PropertyProcessor};
use std::io::Write;

/// WKT according to OpenGIS Simple Features Specification For SQL Revision 1.1
pub struct WktWriter<'a, W: Write> {
    pub dims: CoordDimensions,
    out: &'a mut W,
}

impl<'a, W: Write> WktWriter<'a, W> {
    pub fn new(out: &'a mut W) -> WktWriter<'a, W> {
        WktWriter {
            dims: CoordDimensions::default(),
            out,
        }
    }
}

impl<W: Write> GeomProcessor for WktWriter<'_, W> {
    fn dimensions(&self) -> CoordDimensions {
        self.dims
    }
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        if idx == 0 {
            let _ = self.out.write(&format!("{} {}", x, y).as_bytes())?;
        } else {
            let _ = self.out.write(&format!(", {} {}", x, y).as_bytes())?;
        }
        Ok(())
    }
    fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
        idx: usize,
    ) -> Result<()> {
        if idx == 0 {
            let _ = self.out.write(&format!("{} {}", x, y).as_bytes())?;
        } else {
            let _ = self.out.write(&format!(", {} {}", x, y).as_bytes())?;
        }
        if let Some(z) = z {
            let _ = self.out.write(&format!(" {}", z).as_bytes())?;
        }
        if let Some(m) = m {
            let _ = self.out.write(&format!(" {}", m).as_bytes())?;
        }
        Ok(())
    }

    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        let _ = self.out.write(b"POINT (")?;
        Ok(())
    }
    fn point_end(&mut self, _idx: usize) -> Result<()> {
        let _ = self.out.write(b")")?;
        Ok(())
    }
    fn multipoint_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        let _ = self.out.write(b"MULTIPOINT (")?;
        Ok(())
    }
    fn multipoint_end(&mut self, _idx: usize) -> Result<()> {
        let _ = self.out.write(b")")?;
        Ok(())
    }
    fn linestring_begin(&mut self, tagged: bool, _size: usize, idx: usize) -> Result<()> {
        if tagged {
            let _ = self.out.write(b"LINESTRING (")?;
        } else {
            if idx == 0 {
                let _ = self.out.write(b"(")?;
            } else {
                let _ = self.out.write(b", (")?;
            }
        }
        Ok(())
    }
    fn linestring_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        let _ = self.out.write(b")")?;
        Ok(())
    }
    fn multilinestring_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        let _ = self.out.write(b"MULTILINESTRING (")?;
        Ok(())
    }
    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        let _ = self.out.write(b")")?;
        Ok(())
    }
    fn polygon_begin(&mut self, tagged: bool, _size: usize, idx: usize) -> Result<()> {
        if tagged {
            let _ = self.out.write(b"POLYGON (")?;
            Ok(())
        } else {
            if idx == 0 {
                let _ = self.out.write(b"(")?;
                Ok(())
            } else {
                let _ = self.out.write(b", (")?;
                Ok(())
            }
        }
    }
    fn polygon_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        let _ = self.out.write(b")")?;
        Ok(())
    }
    fn multipolygon_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        let _ = self.out.write(b"MULTIPOLYGON (")?;
        Ok(())
    }
    fn multipolygon_end(&mut self, _idx: usize) -> Result<()> {
        let _ = self.out.write(b")")?;
        Ok(())
    }
    // GEOMETRYCOLLECTION (POINT (10 10),
    // POINT (30 30),
    // LINESTRING (15 15, 20 20))’
}

impl<W: Write> PropertyProcessor for WktWriter<'_, W> {}

impl<W: Write> FeatureProcessor for WktWriter<'_, W> {}
