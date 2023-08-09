use crate::error::{Result, GeozeroError};
use crate::wkb::{WKBByteOrder, WKBGeometryType, WkbDialect};
use crate::{CoordDimensions, FeatureProcessor, GeomProcessor, PropertyProcessor};
use scroll::IOwrite;
use std::io::Write;

/// WKB writer.
pub struct WkbWriter<'a, W: Write> {
    pub dims: CoordDimensions,
    pub srid: Option<i32>,
    /// Geometry envelope (GPKG)
    pub envelope: Vec<f64>,
    /// Envelope dimensions (GPKG)
    pub envelope_dims: CoordDimensions,
    /// ExtendedGeoPackageBinary
    pub extended_gpkg: bool,
    /// Empty geometry flag (GPKG)
    pub empty: bool,
    endian: scroll::Endian,
    dialect: WkbDialect,
    first_header: bool,
    geom_state: GeomState,
    nesting_level: u32,
    out: &'a mut W,
}

#[derive(PartialEq, Debug)]
enum GeomState {
    Normal,
    RingGeom,
    MultiPointGeom,
}

impl<'a, W: Write> WkbWriter<'a, W> {
    pub fn new(out: &'a mut W, dialect: WkbDialect) -> WkbWriter<'a, W> {
        WkbWriter {
            dims: CoordDimensions::default(),
            srid: None,
            envelope: Vec::new(),
            envelope_dims: CoordDimensions::default(),
            extended_gpkg: false,
            empty: false,
            endian: scroll::LE,
            dialect,
            first_header: true,
            geom_state: GeomState::Normal,
            nesting_level: 0,
            out,
        }
    }

    /// Write header in selected format
    fn write_header(&mut self, wkb_type: WKBGeometryType) -> Result<()> {
        match self.dialect {
            WkbDialect::Wkb => self.write_wkb_header(wkb_type),
            WkbDialect::Ewkb => self.write_ewkb_header(wkb_type),
            WkbDialect::Geopackage => {
                if self.first_header {
                    self.write_gpkg_header()?;
                    self.first_header = false;
                }
                self.write_wkb_header(wkb_type)
            }
            WkbDialect::MySQL => {
                if self.first_header {
                    self.write_mysql_header()?;
                    self.first_header = false;
                }
                self.write_wkb_header(wkb_type)
            }
            WkbDialect::Spatialite => self.write_spatialite_header(wkb_type),
        }
    }

    /// OGC WKB header
    fn write_wkb_header(&mut self, wkb_type: WKBGeometryType) -> Result<()> {
        let byte_order: WKBByteOrder = self.endian.into();
        self.out.iowrite(byte_order as u8)?;
        let mut type_id = wkb_type as u32;
        if self.dims.z {
            type_id += 1000;
        }
        if self.dims.m {
            type_id += 2000;
        }
        self.out.iowrite_with(type_id, self.endian)?;
        Ok(())
    }

    /// EWKB header according to https://git.osgeo.org/gitea/postgis/postgis/src/branch/master/doc/ZMSgeoms.txt
    fn write_ewkb_header(&mut self, wkb_type: WKBGeometryType) -> Result<()> {
        let byte_order: WKBByteOrder = self.endian.into();
        self.out.iowrite(byte_order as u8)?;

        let mut type_id = wkb_type as u32;
        if self.dims.z {
            type_id |= 0x8000_0000;
        }
        if self.dims.m {
            type_id |= 0x4000_0000;
        }
        if self.srid.is_some() && self.first_header {
            type_id |= 0x2000_0000;
        }
        self.out.iowrite_with(type_id, self.endian)?;

        if self.first_header {
            // write SRID in main header only
            if let Some(srid) = self.srid {
                self.out.iowrite_with(srid, self.endian)?;
            }
            self.first_header = false;
        }

        Ok(())
    }

    /// GPKG geometry header according to http://www.geopackage.org/spec/#gpb_format
    fn write_gpkg_header(&mut self) -> Result<()> {
        let magic = b"GP";
        self.out.write_all(magic)?;
        let version: u8 = 0;
        self.out.iowrite(version)?;

        let mut flags: u8 = 0;
        if self.extended_gpkg {
            flags |= 0b0010_0000;
        }
        if self.empty {
            flags |= 0b0001_0000;
        }
        let env_info: u8 = if self.envelope.is_empty() {
            0 // no envelope
        } else {
            match (self.envelope_dims.z, self.envelope_dims.m) {
                (false, false) => 1, // [minx, maxx, miny, maxy]
                (true, false) => 2,  // [minx, maxx, miny, maxy, minz, maxz]
                (false, true) => 3,  // [minx, maxx, miny, maxy, minm, maxm]
                (true, true) => 4,   // [minx, maxx, miny, maxy, minz, maxz, minm, maxm]
            }
        };
        flags |= env_info << 1;
        if self.endian == scroll::LE {
            flags |= 0b0000_0001;
        }
        // println!("flags: {flags:#010b}");
        self.out.iowrite(flags)?;

        // srs_id
        // 0: undefined geographic coordinate reference systems
        // -1: undefined Cartesian coordinate reference systems
        self.out.iowrite_with(self.srid.unwrap_or(0), self.endian)?;

        for val in &self.envelope {
            self.out.iowrite_with(*val, self.endian)?;
        }

        Ok(())
    }

    /// Spatialite WKB header according to https://www.gaia-gis.it/gaia-sins/BLOB-Geometry.html
    fn write_spatialite_header(&mut self, wkb_type: WKBGeometryType) -> Result<()> {
        if self.first_header {
            self.out.iowrite(0 as u8)?;
            let byte_order: WKBByteOrder = self.endian.into();
            self.out.iowrite(byte_order as u8)?;
            self.out.iowrite(self.srid.unwrap_or(0))?;
    
            let envelope = Some(&self.envelope).filter(|e| !e.is_empty());
            for val in envelope.unwrap_or(&vec![0.0, 0.0, 0.0, 0.0]) {
                self.out.iowrite_with(*val, self.endian)?;
            }
    
            self.out.iowrite(0x7C as u8)?;
            
            self.first_header = false;
        } else {
            self.out.iowrite(0x69 as u8)?;
        }
        
        let mut type_id = wkb_type as u32;
        if self.dims.z {
            type_id += 1000;
        }
        if self.dims.m {
            type_id += 2000;
        }
        if self.srid.is_some() && self.first_header {
            type_id |= 0x2000_0000;
        }
        self.out.iowrite_with(type_id, self.endian)?;

        Ok(())
    }

    /// MySQL WKB header according to https://dev.mysql.com/doc/refman/8.0/en/gis-data-formats.html
    fn write_mysql_header(&mut self) -> Result<()> {
        let srid: u32 = self.srid.unwrap_or(0)
            .try_into()
            .map_err(|_| GeozeroError::Srid(self.srid.unwrap()))?;
        self.out.iowrite_with(srid, self.endian)?;
        Ok(())
    }
    
    /// Write header in selected format
    fn write_footer(&mut self) -> Result<()> {
        match self.dialect {
            WkbDialect::Wkb => Ok(()),
            WkbDialect::Ewkb => Ok(()),
            WkbDialect::Geopackage => Ok(()),
            WkbDialect::MySQL => Ok(()),
            WkbDialect::Spatialite => {
                if self.nesting_level == 0 {
                    self.out.iowrite(0xFE as u8)?;
                }
                Ok(())
            }
        }
    }

}

impl<W: Write> GeomProcessor for WkbWriter<'_, W> {
    fn dimensions(&self) -> CoordDimensions {
        self.dims
    }
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
        if self.geom_state == GeomState::MultiPointGeom {
            self.write_header(WKBGeometryType::Point)?;
        }
        self.out.iowrite_with(x, self.endian)?;
        self.out.iowrite_with(y, self.endian)?;
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
        _idx: usize,
    ) -> Result<()> {
        if self.geom_state == GeomState::MultiPointGeom {
            self.write_header(WKBGeometryType::Point)?;
        }
        self.out.iowrite_with(x, self.endian)?;
        self.out.iowrite_with(y, self.endian)?;
        if let Some(z) = z {
            self.out.iowrite_with(z, self.endian)?;
        }
        if let Some(m) = m {
            self.out.iowrite_with(m, self.endian)?;
        }
        Ok(())
    }
    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        self.write_header(WKBGeometryType::Point)
    }
    fn point_end(&mut self, _idx: usize) -> Result<()> {
        self.write_footer()
    }
    fn multipoint_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.nesting_level += 1;
        self.write_header(WKBGeometryType::MultiPoint)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        self.geom_state = GeomState::MultiPointGeom;
        Ok(())
    }
    fn multipoint_end(&mut self, _idx: usize) -> Result<()> {
        self.nesting_level -= 1;
        self.geom_state = GeomState::Normal;
        self.write_footer()?;
        Ok(())
    }
    fn linestring_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        if self.geom_state != GeomState::RingGeom {
            self.write_header(WKBGeometryType::LineString)?;
        }
        self.out.iowrite_with(size as u32, self.endian)?;
        Ok(())
    }
    fn linestring_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        self.write_footer()
    }
    fn multilinestring_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.nesting_level += 1;
        self.write_header(WKBGeometryType::MultiLineString)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        Ok(())
    }
    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        self.nesting_level -= 1;
        self.write_footer()?;
        Ok(())
    }
    fn polygon_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        self.write_header(WKBGeometryType::Polygon)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        self.geom_state = GeomState::RingGeom;
        Ok(())
    }
    fn polygon_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        self.geom_state = GeomState::Normal;
        self.write_footer()?;
        Ok(())
    }
    fn multipolygon_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.nesting_level += 1;
        self.write_header(WKBGeometryType::MultiPolygon)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        Ok(())
    }
    fn multipolygon_end(&mut self, _idx: usize) -> Result<()> {
        self.nesting_level -= 1;
        self.write_footer()
    }
    fn geometrycollection_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.nesting_level += 1;
        self.write_header(WKBGeometryType::GeometryCollection)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        Ok(())
    }
    fn geometrycollection_end(&mut self, _idx: usize) -> Result<()> {
        self.nesting_level -= 1;
        self.write_footer()?;
        Ok(())
    }
    fn circularstring_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.write_header(WKBGeometryType::CircularString)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        Ok(())
    }
    fn circularstring_end(&mut self, _idx: usize) -> Result<()> {
        self.write_footer()
    }
    fn compoundcurve_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.write_header(WKBGeometryType::CompoundCurve)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        Ok(())
    }
    fn compoundcurve_end(&mut self, _idx: usize) -> Result<()> {
        self.write_footer()
    }
    fn curvepolygon_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.write_header(WKBGeometryType::CurvePolygon)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        Ok(())
    }
    fn curvepolygon_end(&mut self, _idx: usize) -> Result<()> {
        self.write_footer()
    }
    fn multicurve_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.nesting_level += 1;
        self.write_header(WKBGeometryType::MultiCurve)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        Ok(())
    }
    fn multicurve_end(&mut self, _idx: usize) -> Result<()> {
        self.nesting_level -= 1;
        self.write_footer()?;
        Ok(())
    }
    fn multisurface_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.nesting_level += 1;
        self.write_header(WKBGeometryType::MultiSurface)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        Ok(())
    }
    fn multisurface_end(&mut self, _idx: usize) -> Result<()> {
        self.nesting_level -= 1;
        self.write_footer()?;
        Ok(())
    }
    fn triangle_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        self.write_header(WKBGeometryType::Triangle)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        self.geom_state = GeomState::RingGeom;
        Ok(())
    }
    fn triangle_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        self.geom_state = GeomState::Normal;
        self.write_footer()?;
        Ok(())
    }
    fn polyhedralsurface_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.write_header(WKBGeometryType::PolyhedralSurface)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        Ok(())
    }
    fn polyhedralsurface_end(&mut self, _idx: usize) -> Result<()> {
        self.write_footer()
    }
    fn tin_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.write_header(WKBGeometryType::Tin)?;
        self.out.iowrite_with(size as u32, self.endian)?;
        Ok(())
    }
    fn tin_end(&mut self, _idx: usize) -> Result<()> {
        self.write_footer()
    }
}

impl<W: Write> PropertyProcessor for WkbWriter<'_, W> {}

impl<W: Write> FeatureProcessor for WkbWriter<'_, W> {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::wkb::process_wkb_type_geom;
    use crate::ToWkb;

    fn roundtrip(
        dialect: WkbDialect,
        ewkb_str: &str,
        dims: CoordDimensions,
        srid: Option<i32>,
        envelope: Vec<f64>,
    ) {
        let wkb_in = hex::decode(ewkb_str).unwrap();
        let mut wkb_out: Vec<u8> = Vec::new();
        let mut writer = WkbWriter::new(&mut wkb_out, dialect);
        writer.dims = dims;
        writer.srid = srid;
        writer.envelope = envelope;
        assert!(process_wkb_type_geom(&mut wkb_in.as_slice(), &mut writer, dialect).is_ok());

        assert_eq!(hex::encode(wkb_in), hex::encode(wkb_out));
    }

    #[test]
    fn ewkb_geometries() {
        // SELECT 'POINT(10 -20)'::geometry
        roundtrip(WkbDialect::Ewkb, "0101000000000000000000244000000000000034C0",
        CoordDimensions::default(), None, Vec::new());

        // SELECT 'SRID=4326;MULTIPOINT (10 -20 100, 0 -0.5 101)'::geometry
        roundtrip(WkbDialect::Ewkb, "01040000A0E6100000020000000101000080000000000000244000000000000034C0000000000000594001010000800000000000000000000000000000E0BF0000000000405940",
        CoordDimensions::xyz(), Some(4326), Vec::new());

        // SELECT 'SRID=4326;LINESTRING (10 -20 100, 0 -0.5 101)'::geometry
        roundtrip(WkbDialect::Ewkb, "01020000A0E610000002000000000000000000244000000000000034C000000000000059400000000000000000000000000000E0BF0000000000405940",
        CoordDimensions::xyz(), Some(4326), Vec::new());

        // SELECT 'SRID=4326;MULTILINESTRING ((10 -20, 0 -0.5), (0 0, 2 0))'::geometry
        roundtrip(WkbDialect::Ewkb, "0105000020E610000002000000010200000002000000000000000000244000000000000034C00000000000000000000000000000E0BF0102000000020000000000000000000000000000000000000000000000000000400000000000000000",
        CoordDimensions::default(), Some(4326), Vec::new());

        // SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry
        roundtrip(WkbDialect::Ewkb, "0103000020E610000001000000050000000000000000000000000000000000000000000000000000400000000000000000000000000000004000000000000000400000000000000000000000000000004000000000000000000000000000000000",
        CoordDimensions::default(), Some(4326), Vec::new());

        // SELECT 'SRID=4326;MULTIPOLYGON (((0 0, 2 0, 2 2, 0 2, 0 0)), ((10 10, -2 10, -2 -2, 10 -2, 10 10)))'::geometry
        roundtrip(WkbDialect::Ewkb, "0106000020E610000002000000010300000001000000050000000000000000000000000000000000000000000000000000400000000000000000000000000000004000000000000000400000000000000000000000000000004000000000000000000000000000000000010300000001000000050000000000000000002440000000000000244000000000000000C0000000000000244000000000000000C000000000000000C0000000000000244000000000000000C000000000000024400000000000002440",
        CoordDimensions::default(), Some(4326), Vec::new());

        // SELECT 'GeometryCollection(POINT (10 10),POINT (30 30),LINESTRING (15 15, 20 20))'::geometry
        roundtrip(WkbDialect::Ewkb, "01070000000300000001010000000000000000002440000000000000244001010000000000000000003E400000000000003E400102000000020000000000000000002E400000000000002E4000000000000034400000000000003440",
        CoordDimensions::default(), None, Vec::new());
    }

    #[test]
    fn ewkb_curves() {
        // SELECT 'CIRCULARSTRING(0 0,1 1,2 0)'::geometry
        roundtrip(WkbDialect::Ewkb, "01080000000300000000000000000000000000000000000000000000000000F03F000000000000F03F00000000000000400000000000000000",
        CoordDimensions::default(), None, Vec::new());

        // SELECT 'COMPOUNDCURVE (CIRCULARSTRING (0 0,1 1,2 0),(2 0,3 0))'::geometry
        roundtrip(WkbDialect::Ewkb, "01090000000200000001080000000300000000000000000000000000000000000000000000000000F03F000000000000F03F000000000000004000000000000000000102000000020000000000000000000040000000000000000000000000000008400000000000000000",
        CoordDimensions::default(), None, Vec::new());

        // SELECT 'CURVEPOLYGON(COMPOUNDCURVE(CIRCULARSTRING(0 0,1 1,2 0),(2 0,3 0,3 -1,0 -1,0 0)))'::geometry
        roundtrip(WkbDialect::Ewkb, "010A0000000100000001090000000200000001080000000300000000000000000000000000000000000000000000000000F03F000000000000F03F0000000000000040000000000000000001020000000500000000000000000000400000000000000000000000000000084000000000000000000000000000000840000000000000F0BF0000000000000000000000000000F0BF00000000000000000000000000000000",
        CoordDimensions::default(), None, Vec::new());

        // SELECT 'MULTICURVE((0 0, 5 5),CIRCULARSTRING(4 0, 4 4, 8 4))'::geometry
        roundtrip(WkbDialect::Ewkb, "010B000000020000000102000000020000000000000000000000000000000000000000000000000014400000000000001440010800000003000000000000000000104000000000000000000000000000001040000000000000104000000000000020400000000000001040",
        CoordDimensions::default(), None, Vec::new());

        // SELECT 'MULTISURFACE (CURVEPOLYGON (COMPOUNDCURVE (CIRCULARSTRING (0 0,1 1,2 0),(2 0,3 0,3 -1,0 -1,0 0))))'::geometry
        roundtrip(WkbDialect::Ewkb, "010C00000001000000010A0000000100000001090000000200000001080000000300000000000000000000000000000000000000000000000000F03F000000000000F03F0000000000000040000000000000000001020000000500000000000000000000400000000000000000000000000000084000000000000000000000000000000840000000000000F0BF0000000000000000000000000000F0BF00000000000000000000000000000000",
        CoordDimensions::default(), None, Vec::new());
    }

    #[test]
    fn ewkb_surfaces() {
        // SELECT 'POLYHEDRALSURFACE(((0 0 0,0 0 1,0 1 1,0 1 0,0 0 0)),((0 0 0,0 1 0,1 1 0,1 0 0,0 0 0)),((0 0 0,1 0 0,1 0 1,0 0 1,0 0 0)),((1 1 0,1 1 1,1 0 1,1 0 0,1 1 0)),((0 1 0,0 1 1,1 1 1,1 1 0,0 1 0)),((0 0 1,1 0 1,1 1 1,0 1 1,0 0 1)))'::geometry
        roundtrip(WkbDialect::Ewkb, "010F000080060000000103000080010000000500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F000000000000F03F0000000000000000000000000000F03F0000000000000000000000000000000000000000000000000000000000000000010300008001000000050000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F000000000000F03F0000000000000000000000000000F03F0000000000000000000000000000000000000000000000000000000000000000000000000000000001030000800100000005000000000000000000000000000000000000000000000000000000000000000000F03F00000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F00000000000000000000000000000000000000000000F03F00000000000000000000000000000000000000000000000001030000800100000005000000000000000000F03F000000000000F03F0000000000000000000000000000F03F000000000000F03F000000000000F03F000000000000F03F0000000000000000000000000000F03F000000000000F03F00000000000000000000000000000000000000000000F03F000000000000F03F0000000000000000010300008001000000050000000000000000000000000000000000F03F00000000000000000000000000000000000000000000F03F000000000000F03F000000000000F03F000000000000F03F000000000000F03F000000000000F03F000000000000F03F00000000000000000000000000000000000000000000F03F00000000000000000103000080010000000500000000000000000000000000000000000000000000000000F03F000000000000F03F0000000000000000000000000000F03F000000000000F03F000000000000F03F000000000000F03F0000000000000000000000000000F03F000000000000F03F00000000000000000000000000000000000000000000F03F",
        CoordDimensions::xyz(), None, Vec::new());

        // SELECT 'TIN(((0 0 0,0 0 1,0 1 0,0 0 0)),((0 0 0,0 1 0,1 1 0,0 0 0)))'::geometry
        roundtrip(WkbDialect::Ewkb, "0110000080020000000111000080010000000400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F0000000000000000000000000000000000000000000000000000000000000000011100008001000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000F03F0000000000000000000000000000F03F000000000000F03F0000000000000000000000000000000000000000000000000000000000000000",
        CoordDimensions::xyz(), None, Vec::new());

        // SELECT 'TRIANGLE((0 0,0 9,9 0,0 0))'::geometry
        roundtrip(WkbDialect::Ewkb, "0111000000010000000400000000000000000000000000000000000000000000000000000000000000000022400000000000002240000000000000000000000000000000000000000000000000",
        CoordDimensions::default(), None, Vec::new());
    }

    #[test]
    fn gpkg_geometries() {
        // pt2d
        roundtrip(WkbDialect::Geopackage, "47500003E61000009A9999999999F13F9A9999999999F13F9A9999999999F13F9A9999999999F13F01010000009A9999999999F13F9A9999999999F13F",
            CoordDimensions::default(), Some(4326), vec![1.1, 1.1, 1.1, 1.1]);

        // mln3dzm
        roundtrip(WkbDialect::Geopackage, "47500003E6100000000000000000244000000000000034400000000000002440000000000000344001BD0B00000100000001BA0B0000020000000000000000003440000000000000244000000000000008400000000000001440000000000000244000000000000034400000000000001C400000000000000040",
            CoordDimensions::xyzm(), Some(4326), vec![10.0, 20.0, 10.0, 20.0]);

        // gc2d
        roundtrip(WkbDialect::Geopackage, "47500003e6100000000000000000f03f0000000000003640000000000000084000000000000036400107000000020000000101000000000000000000f03f00000000000008400103000000010000000400000000000000000035400000000000003540000000000000364000000000000035400000000000003540000000000000364000000000000035400000000000003540",
            CoordDimensions::default(), Some(4326), vec![1.0, 22.0, 3.0, 22.0]);
    }

    #[test]
    fn spatialite_geometries() {
        // SELECT HEX(ST_GeomFromText('POINT(1.1 1.1)', 4326));
        roundtrip(WkbDialect::Spatialite, "0001E61000009A9999999999F13F9A9999999999F13F9A9999999999F13F9A9999999999F13F7C010000009A9999999999F13F9A9999999999F13FFE",
            CoordDimensions::default(), Some(4326), vec![1.1, 1.1, 1.1, 1.1]);

        // SELECT HEX(ST_GeomFromText('MULTIPOINT(1 2,3 4)'));
        roundtrip(WkbDialect::Spatialite, "000100000000000000000000F03F0000000000000040000000000000084000000000000010407C04000000020000006901000000000000000000F03F0000000000000040690100000000000000000008400000000000001040FE",
        CoordDimensions::default(), None, vec![1.0, 2.0, 3.0, 4.0]);
            
        // SELECT HEX(ST_GeomFromText('MULTILINESTRINGZM((20 10 5 1,10 20 30 40))'));
        roundtrip(WkbDialect::Spatialite, "00010000000000000000000024400000000000002440000000000000344000000000000034407CBD0B00000100000069BA0B000002000000000000000000344000000000000024400000000000001440000000000000F03F000000000000244000000000000034400000000000003E400000000000004440FE",
            CoordDimensions::xyzm(), None, vec![10.0, 10.0, 20.0, 20.0]);

        // SELECT HEX(ST_GeomFromText('GEOMETRYCOLLECTION(POINT(1 3),POLYGON((21 21,22 21,21 22,21 21)))'));
        roundtrip(WkbDialect::Spatialite, "000100000000000000000000F03F0000000000000840000000000000364000000000000036407C07000000020000006901000000000000000000F03F00000000000008406903000000010000000400000000000000000035400000000000003540000000000000364000000000000035400000000000003540000000000000364000000000000035400000000000003540FE",
            CoordDimensions::default(), None, vec![1.0, 3.0, 22.0, 22.0]);
    }

    #[test]
    fn mysql_geometries() {
        // SELECT HEX(ST_GeomFromText('POINT(10 -20)', 4326, 'axis-order=long-lat'));
        roundtrip(WkbDialect::MySQL, "E61000000101000000000000000000244000000000000034C0",
            CoordDimensions::default(), Some(4326), Vec::new());

        // SELECT HEX(ST_GeomFromText('MULTIPOINT(1 2,3 4)', 0, 'axis-order=long-lat'));
        roundtrip(WkbDialect::MySQL, "000000000104000000020000000101000000000000000000F03F0000000000000040010100000000000000000008400000000000001040",
        CoordDimensions::default(), None, Vec::new());
            
        // SELECT HEX(ST_GeomFromText('MULTILINESTRING((20 10,10 20))', 0, 'axis-order=long-lat'));
        roundtrip(WkbDialect::MySQL, "000000000105000000010000000102000000020000000000000000003440000000000000244000000000000024400000000000003440",
            CoordDimensions::default(), None, Vec::new());

        // SELECT HEX(ST_GeomFromText('GEOMETRYCOLLECTION(POINT(1 3),POLYGON((21 21,22 21,21 22,21 21)))', 0, 'axis-order=long-lat'));
        roundtrip(WkbDialect::MySQL, "000000000107000000020000000101000000000000000000F03F00000000000008400103000000010000000400000000000000000035400000000000003540000000000000364000000000000035400000000000003540000000000000364000000000000035400000000000003540",
            CoordDimensions::default(), None, Vec::new());
    }

    #[test]
    #[cfg(feature = "with-geo")]
    fn conversions() {
        let geom: geo_types::Geometry<f64> = geo_types::Point::new(10.0, -20.0).into();
        let wkb = geom.to_ewkb(CoordDimensions::default(), None).unwrap();
        assert_eq!(
            &wkb,
            // SELECT 'POINT(10 -20)'::geometry
            &hex::decode("0101000000000000000000244000000000000034C0").unwrap()
        );
        let wkb = geom
            .to_ewkb(CoordDimensions::default(), Some(4326))
            .unwrap();
        assert_eq!(
            &wkb,
            &[1, 1, 0, 0, 32, 230, 16, 0, 0, 0, 0, 0, 0, 0, 0, 36, 64, 0, 0, 0, 0, 0, 0, 52, 192]
        );

        let geom: geo_types::Geometry<f64> = geo_types::Point::new(1.1, 1.1).into();
        let wkb = geom
            .to_gpkg_wkb(
                CoordDimensions::default(),
                Some(4326),
                vec![1.1, 1.1, 1.1, 1.1],
            )
            .unwrap();
        assert_eq!(
            &wkb,
            &hex::decode("47500003E61000009A9999999999F13F9A9999999999F13F9A9999999999F13F9A9999999999F13F01010000009A9999999999F13F9A9999999999F13F").unwrap()
        );

        let geom: geo_types::Geometry<f64> = geo_types::Point::new(1.1, 1.1).into();
        let wkb = geom
            .to_spatialite_wkb(
                CoordDimensions::default(),
                Some(4326),
                vec![1.1, 1.1, 1.1, 1.1],
            )
            .unwrap();
        assert_eq!(
            &wkb,
            &hex::decode("0001E61000009A9999999999F13F9A9999999999F13F9A9999999999F13F9A9999999999F13F7C010000009A9999999999F13F9A9999999999F13FFE").unwrap()
        );

        let geom: geo_types::Geometry<f64> = geo_types::Point::new(10.0, -20.0).into();
        let wkb = geom
            .to_mysql_wkb(Some(4326))
            .unwrap();
        assert_eq!(
            &wkb,
            &hex::decode("E61000000101000000000000000000244000000000000034C0").unwrap()
        );
    }
}
