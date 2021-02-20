use crate::error::Result;
use crate::{CoordDimensions, FeatureProcessor, GeomProcessor, PropertyProcessor};
use std::io::Write;

/// WKT Writer.
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
    fn geom_begin(&mut self, idx: usize, tag: &[u8]) -> Result<()> {
        if idx > 0 {
            let _ = self.out.write(b",")?;
        }
        let _ = self.out.write(tag)?;
        Ok(())
    }
    fn tagged_geom_begin(&mut self, tagged: bool, idx: usize, tag: &[u8]) -> Result<()> {
        if idx > 0 {
            let _ = self.out.write(b",")?;
        }
        if tagged {
            let _ = self.out.write(tag)?;
        } else {
            let _ = self.out.write(b"(")?;
        }
        Ok(())
    }
    fn geom_end(&mut self) -> Result<()> {
        let _ = self.out.write(b")")?;
        Ok(())
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
            let _ = self.out.write(&format!(",{} {}", x, y).as_bytes())?;
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
            let _ = self.out.write(&format!(",{} {}", x, y).as_bytes())?;
        }
        if let Some(z) = z {
            let _ = self.out.write(&format!(" {}", z).as_bytes())?;
        }
        if let Some(m) = m {
            let _ = self.out.write(&format!(" {}", m).as_bytes())?;
        }
        Ok(())
    }

    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.geom_begin(idx, b"POINT(")
    }
    fn point_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn multipoint_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.geom_begin(idx, b"MULTIPOINT(")
    }
    fn multipoint_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn linestring_begin(&mut self, tagged: bool, _size: usize, idx: usize) -> Result<()> {
        self.tagged_geom_begin(tagged, idx, b"LINESTRING(")
    }
    fn linestring_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn multilinestring_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.geom_begin(idx, b"MULTILINESTRING(")
    }
    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn polygon_begin(&mut self, tagged: bool, _size: usize, idx: usize) -> Result<()> {
        self.tagged_geom_begin(tagged, idx, b"POLYGON(")
    }
    fn polygon_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn multipolygon_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.geom_begin(idx, b"MULTIPOLYGON(")
    }
    fn multipolygon_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn geometrycollection_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        let _ = self.out.write(b"GEOMETRYCOLLECTION(")?;
        Ok(())
    }
    fn geometrycollection_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn circularstring_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.geom_begin(idx, b"CIRCULARSTRING(")
    }
    fn circularstring_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn compoundcurve_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.geom_begin(idx, b"COMPOUNDCURVE(")
    }

    fn compoundcurve_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn curvepolygon_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.geom_begin(idx, b"CURVEPOLYGON(")
    }
    fn curvepolygon_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn multicurve_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.geom_begin(idx, b"MULTICURVE(")
    }
    fn multicurve_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn multisurface_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.geom_begin(idx, b"MULTISURFACE(")
    }
    fn multisurface_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }

    fn triangle_begin(&mut self, tagged: bool, _size: usize, idx: usize) -> Result<()> {
        self.tagged_geom_begin(tagged, idx, b"TRIANGLE(")
    }
    fn triangle_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn polyhedralsurface_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.geom_begin(idx, b"POLYHEDRALSURFACE(")
    }
    fn polyhedralsurface_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn tin_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.geom_begin(idx, b"TIN(")
    }
    fn tin_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
}

impl<W: Write> PropertyProcessor for WktWriter<'_, W> {}

impl<W: Write> FeatureProcessor for WktWriter<'_, W> {}

pub(crate) mod conversion {
    use super::*;
    use crate::{GeozeroGeometry, GeozeroGeometryReader};
    use std::io::Read;

    /// Convert to WKT.
    pub trait ToWkt {
        /// Convert to 2D WKT String.
        fn to_wkt(&self) -> Result<String>;
        /// Convert to WKT String with dimensions.
        fn to_wkt_ndim(&self, dims: CoordDimensions) -> Result<String>;
    }

    impl<T: GeozeroGeometry> ToWkt for T {
        fn to_wkt(&self) -> Result<String> {
            self.to_wkt_ndim(CoordDimensions::default())
        }
        fn to_wkt_ndim(&self, dims: CoordDimensions) -> Result<String> {
            let mut out: Vec<u8> = Vec::new();
            let mut writer = WktWriter::new(&mut out);
            writer.dims = dims;
            T::process_geom(self, &mut writer)?;
            String::from_utf8(out).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })
        }
    }

    pub trait ReadAsWkt {
        /// Read to 2D WKT String.
        fn read_as_wkt<R: Read>(reader: R) -> Result<String>;
        /// Read to WKT String with dimensions.
        fn read_as_wkt_ndim<R: Read>(reader: R, dims: CoordDimensions) -> Result<String>;
    }

    impl<T: GeozeroGeometryReader> ReadAsWkt for T {
        fn read_as_wkt<R: Read>(reader: R) -> Result<String> {
            Self::read_as_wkt_ndim(reader, CoordDimensions::default())
        }
        fn read_as_wkt_ndim<R: Read>(reader: R, dims: CoordDimensions) -> Result<String> {
            let mut out: Vec<u8> = Vec::new();
            let mut writer = WktWriter::new(&mut out);
            writer.dims = dims;
            T::read_geom(reader, &mut writer)?;
            String::from_utf8(out).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::conversion::*;

    #[test]
    fn to_wkt() {
        let geom: geo_types::Geometry<f64> = geo_types::Point::new(10.0, 20.0).into();
        assert_eq!(&geom.to_wkt().unwrap(), "POINT(10 20)");
    }
}
