use crate::error::Result;
use crate::{CoordDimensions, FeatureProcessor, GeomProcessor, PropertyProcessor};
use std::io::Write;
use std::vec;

use super::WktDialect;

/// WKT Writer.
pub struct WktWriter<W: Write> {
    dims: CoordDimensions,
    srid: Option<i32>,
    dialect: WktDialect,
    first_header: bool,
    /// Stack of in-progress geometry sizes
    geometry_sizes: Vec<usize>,
    pub(crate) out: W,
}

impl<W: Write> WktWriter<W> {
    pub fn new(out: W) -> Self {
        Self::with_opts(out, WktDialect::Wkt, CoordDimensions::default(), None)
    }

    pub fn with_dims(out: W, dims: CoordDimensions) -> Self {
        Self::with_opts(out, WktDialect::Wkt, dims, None)
    }

    pub fn with_opts(
        out: W,
        dialect: WktDialect,
        dims: CoordDimensions,
        srid: Option<i32>,
    ) -> Self {
        Self {
            dims,
            srid,
            dialect,
            first_header: true,
            geometry_sizes: vec![],
            out,
        }
    }

    fn header(&mut self, srid: Option<i32>) -> Result<()> {
        if self.first_header && self.dialect == WktDialect::Ewkt {
            self.first_header = false;
            match srid {
                None | Some(0) => (),
                Some(srid) => self.out.write_all(format!("SRID={srid};").as_bytes())?,
            }
        }
        Ok(())
    }
    fn comma(&mut self, idx: usize) -> Result<()> {
        if idx > 0 {
            self.out.write_all(b",")?;
        }
        Ok(())
    }
    fn geom_begin(&mut self, tag: &[u8], tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.header(self.srid)?;
        self.comma(idx)?;
        if tagged {
            self.out.write_all(tag)?;
        }
        self.geometry_sizes.push(size);
        if size == 0 {
            if tagged {
                self.out.write_all(b" ")?;
            };
            self.out.write_all(b"EMPTY")?;
        } else {
            self.out.write_all(b"(")?;
        }
        Ok(())
    }
    fn geom_end(&mut self) -> Result<()> {
        if let Some(geometry_size) = self.geometry_sizes.pop() {
            if geometry_size > 0 {
                self.out.write_all(b")")?;
            }
        } else {
            debug_assert!(false, "ended geometry that didn't start");
        }
        Ok(())
    }
}

impl<W: Write> GeomProcessor for WktWriter<W> {
    fn dimensions(&self) -> CoordDimensions {
        self.dims
    }

    fn srid(&mut self, srid: Option<i32>) -> Result<()> {
        self.srid = self.srid.or(srid);
        Ok(())
    }

    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        self.comma(idx)?;
        if f64::is_nan(x) && f64::is_nan(y) {
            self.out.write_all(b"EMPTY")?;
        } else {
            self.out.write_all(format!("{x} {y}").as_bytes())?;
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
        self.comma(idx)?;
        if f64::is_nan(x)
            && f64::is_nan(y)
            && z.map(f64::is_nan).unwrap_or(true)
            && m.map(f64::is_nan).unwrap_or(true)
        {
            self.out.write_all(b"EMPTY")?;
        } else {
            self.out.write_all(format!("{x} {y}").as_bytes())?;
            if let Some(z) = z {
                self.out.write_all(format!(" {z}").as_bytes())?;
            }
            if let Some(m) = m {
                self.out.write_all(format!(" {m}").as_bytes())?;
            }
        }
        Ok(())
    }

    fn empty_point(&mut self, idx: usize) -> Result<()> {
        self.geom_begin(b"POINT", true, 0, idx)?;
        self.geom_end()
    }
    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.geom_begin(b"POINT", true, 1, idx)
    }
    fn point_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"MULTIPOINT", true, size, idx)
    }
    fn multipoint_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"LINESTRING", tagged, size, idx)
    }
    fn linestring_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"MULTILINESTRING", true, size, idx)
    }
    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"POLYGON", tagged, size, idx)
    }
    fn polygon_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"MULTIPOLYGON", true, size, idx)
    }
    fn multipolygon_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"GEOMETRYCOLLECTION", true, size, idx)
    }
    fn geometrycollection_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn circularstring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"CIRCULARSTRING", true, size, idx)
    }
    fn circularstring_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"COMPOUNDCURVE", true, size, idx)
    }
    fn compoundcurve_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"CURVEPOLYGON", true, size, idx)
    }
    fn curvepolygon_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn multicurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"MULTICURVE", true, size, idx)
    }
    fn multicurve_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn multisurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"MULTISURFACE", true, size, idx)
    }
    fn multisurface_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn triangle_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"TRIANGLE", tagged, size, idx)
    }
    fn triangle_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"POLYHEDRALSURFACE", true, size, idx)
    }
    fn polyhedralsurface_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
    fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom_begin(b"TIN", true, size, idx)
    }
    fn tin_end(&mut self, _idx: usize) -> Result<()> {
        self.geom_end()
    }
}

impl<W: Write> PropertyProcessor for WktWriter<W> {}

impl<W: Write> FeatureProcessor for WktWriter<W> {}

#[cfg(test)]
mod test {
    #[cfg(feature = "with-wkb")]
    use crate::wkb::{FromWkb, WkbDialect};
    #[cfg(feature = "with-wkb")]
    use crate::wkt::EwktString;
    use crate::ToWkt;

    #[test]
    #[cfg(feature = "with-geo")]
    fn to_wkt() {
        let geom: geo_types::Geometry<f64> = geo_types::Point::new(10.0, 20.0).into();
        assert_eq!(&geom.to_wkt().unwrap(), "POINT(10 20)");
        assert_eq!(&geom.to_ewkt(Some(4326)).unwrap(), "SRID=4326;POINT(10 20)");
    }

    #[test]
    #[cfg(feature = "with-wkb")]
    fn from_wkb() {
        let blob = hex::decode("01040000A0E6100000020000000101000080000000000000244000000000000034C0000000000000594001010000800000000000000000000000000000E0BF0000000000405940").unwrap();
        let mut cursor = std::io::Cursor::new(blob);
        let ewkt = EwktString::from_wkb(&mut cursor, WkbDialect::Ewkb).unwrap();
        assert_eq!(ewkt.0, "SRID=4326;MULTIPOINT(10 -20 100,0 -0.5 101)")
    }
}
