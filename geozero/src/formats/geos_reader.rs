use crate::error::{GeozeroError, Result};
use crate::{CoordDimensions, GeomProcessor, GeozeroGeometry};
use geos::{CoordSeq, Geom, Geometry as GGeometry, GeometryTypes};

pub(crate) fn from_geos_err(error: geos::Error) -> GeozeroError {
    match error {
        geos::Error::InvalidGeometry(e) => GeozeroError::Geometry(e),
        geos::Error::ImpossibleOperation(e) => GeozeroError::Geometry(e),
        geos::Error::GeosError(e) => GeozeroError::Geometry(e),
        geos::Error::GeosFunctionError(_, _) => GeozeroError::GeometryFormat,
        geos::Error::NoConstructionFromNullPtr(e) => GeozeroError::Geometry(e),
        geos::Error::ConversionError(e) => GeozeroError::Geometry(e),
        geos::Error::GenericError(e) => GeozeroError::Geometry(e),
    }
}

/// Process GEOS geometry.
pub fn process_geom<P: GeomProcessor>(ggeom: &GGeometry, processor: &mut P) -> Result<()> {
    process_geom_n(ggeom, 0, processor)
}

fn process_geom_n<'a, P: GeomProcessor, G: Geom<'a>>(
    ggeom: &G,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    match ggeom.geometry_type() {
        GeometryTypes::Point => {
            processor.point_begin(idx)?;
            process_point(ggeom, 0, processor)?;
            processor.point_end(idx)?;
        }
        GeometryTypes::MultiPoint => {
            let n_pts = ggeom.get_num_geometries().map_err(from_geos_err)?;
            processor.multipoint_begin(n_pts, idx)?;
            for i in 0..n_pts {
                let pt = ggeom.get_geometry_n(i).map_err(from_geos_err)?;
                process_point(&pt, i, processor)?;
            }
            processor.multipoint_end(idx)?;
        }
        GeometryTypes::LineString | GeometryTypes::LinearRing => {
            process_linestring(ggeom, true, idx, processor)?;
        }
        GeometryTypes::MultiLineString => {
            let n_lines = ggeom.get_num_geometries().map_err(from_geos_err)?;
            processor.multilinestring_begin(n_lines, idx)?;
            for i in 0..n_lines {
                let line = ggeom.get_geometry_n(i).map_err(from_geos_err)?;
                process_linestring(&line, false, i, processor)?;
            }
            processor.multilinestring_end(idx)?;
        }
        GeometryTypes::Polygon => {
            process_polygon(ggeom, true, idx, processor)?;
        }
        GeometryTypes::MultiPolygon => {
            let n_polys = ggeom.get_num_geometries().map_err(from_geos_err)?;
            processor.multipolygon_begin(n_polys, idx)?;
            for i in 0..n_polys {
                let poly = ggeom.get_geometry_n(i).map_err(from_geos_err)?;
                process_polygon(&poly, false, i, processor)?;
            }
            processor.multipolygon_end(idx)?;
        }
        GeometryTypes::GeometryCollection => {
            let n_geoms = ggeom.get_num_geometries().map_err(from_geos_err)?;
            processor.geometrycollection_begin(n_geoms, idx)?;
            for i in 0..n_geoms {
                let g = ggeom.get_geometry_n(i).map_err(from_geos_err)?;
                process_geom_n(&g, i, processor)?;
            }
            processor.geometrycollection_end(idx)?;
        }
        GeometryTypes::__Unknonwn(_) => return Err(GeozeroError::GeometryFormat),
    }
    Ok(())
}

fn process_coord_seq<P: GeomProcessor>(
    cs: &CoordSeq,
    offset: usize,
    processor: &mut P,
) -> Result<()> {
    let multi = processor.dimensions().z;
    let n_coords = cs.size().map_err(from_geos_err)?;
    for i in 0..n_coords {
        if multi {
            processor.coordinate(
                cs.get_x(i).map_err(from_geos_err)?,
                cs.get_y(i).map_err(from_geos_err)?,
                Some(cs.get_z(i).map_err(from_geos_err)?),
                None,
                None,
                None,
                offset + i,
            )?;
        } else {
            processor.xy(
                cs.get_x(i).map_err(from_geos_err)?,
                cs.get_y(i).map_err(from_geos_err)?,
                offset + i, // multipoints have offset > 0, but i is always 0
            )?;
        }
    }
    Ok(())
}

fn process_point<'a, P: GeomProcessor, G: Geom<'a>>(
    ggeom: &G,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let cs = ggeom.get_coord_seq().map_err(from_geos_err)?;
    // NOTE: this clones the underlying CoordSeq!
    // let x = GEOSGeom_getX_r(ggeom.get_raw_context(), ggeom.as_raw());
    process_coord_seq(&cs, idx, processor)?;
    Ok(())
}

fn process_linestring<'a, P: GeomProcessor, G: Geom<'a>>(
    ggeom: &G,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let cs = ggeom.get_coord_seq().map_err(from_geos_err)?;
    // NOTE: this clones the underlying CoordSeq!
    // let coords = GEOSGeom_getCoordSeq_r(ggeom.get_raw_context(), ggeom.as_raw());
    let n_coords = cs.size().map_err(from_geos_err)?;
    processor.linestring_begin(tagged, n_coords, idx)?;
    process_coord_seq(&cs, 0, processor)?;
    processor.linestring_end(tagged, idx)
}

fn process_polygon<'a, P: GeomProcessor, G: Geom<'a>>(
    ggeom: &G,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let nb_interiors = ggeom.get_num_interior_rings().map_err(from_geos_err)?;

    processor.polygon_begin(tagged, nb_interiors + 1, idx)?;
    // Exterior ring
    let ring = ggeom.get_exterior_ring().map_err(from_geos_err)?;
    process_linestring(&ring, false, 0, processor)?;
    // Interior rings
    for ix_interior in 0..nb_interiors {
        let ring = ggeom
            .get_interior_ring_n(ix_interior as u32)
            .map_err(from_geos_err)?;
        process_linestring(&ring, false, ix_interior + 1, processor)?;
    }
    processor.polygon_end(tagged, idx)
}

// --- impl conversion traits

impl GeozeroGeometry for geos::Geometry<'_> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        process_geom(self, processor)
    }
    fn empty() -> Self {
        GGeometry::create_empty_point().unwrap()
    }
    fn dims(&self) -> CoordDimensions {
        CoordDimensions {
            z: self.has_z().unwrap_or(false),
            ..Default::default()
        }
    }
    fn srid(&self) -> Option<i32> {
        self.get_srid().map(|srid| srid as i32).ok()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::wkt_writer::WktWriter;

    #[test]
    fn point_geom() {
        let wkt = "POINT(1 1)";
        let ggeom = GGeometry::new_from_wkt(wkt).unwrap();

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_geom(&ggeom, &mut WktWriter::new(&mut wkt_data)).is_ok());

        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), wkt);
    }

    #[test]
    fn multipoint_geom() {
        let wkt = "MULTIPOINT(1 1,2 2)";
        // let geos_wkt = "MULTIPOINT((1 1),(2 2))";
        let ggeom = GGeometry::new_from_wkt(wkt).unwrap();

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_geom(&ggeom, &mut WktWriter::new(&mut wkt_data)).is_ok());

        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), wkt);
    }

    #[test]
    fn line_geom() {
        let wkt = "LINESTRING(1 1,2 2)";
        let ggeom = GGeometry::new_from_wkt(wkt).unwrap();

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_geom(&ggeom, &mut WktWriter::new(&mut wkt_data)).is_ok());

        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), wkt);
    }

    #[test]
    fn line_geom_3d() {
        let wkt = "LINESTRING(1 1 10,2 2 20)";
        let ggeom = GGeometry::new_from_wkt(wkt).unwrap();

        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        writer.dims.z = true;
        assert!(process_geom(&ggeom, &mut writer).is_ok());

        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), wkt);
    }

    #[test]
    fn linearring_geom() {
        let wkt = "LINEARRING(1 1,2 1,2 2,1 1)";
        let ggeom = GGeometry::new_from_wkt(wkt).unwrap();

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_geom(&ggeom, &mut WktWriter::new(&mut wkt_data)).is_ok());

        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "LINESTRING(1 1,2 1,2 2,1 1)"
        );
    }

    #[test]
    fn multiline_geom() {
        let wkt = "MULTILINESTRING((1 1,2 2),(3 3,4 4))";
        let ggeom = GGeometry::new_from_wkt(wkt).unwrap();

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_geom(&ggeom, &mut WktWriter::new(&mut wkt_data)).is_ok());

        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), wkt);
    }

    #[test]
    fn polygon_geom() {
        let wkt = "POLYGON((0 0,0 3,3 3,3 0,0 0),(0.2 0.2,0.2 2,2 2,2 0.2,0.2 0.2))";
        let ggeom = GGeometry::new_from_wkt(wkt).unwrap();

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_geom(&ggeom, &mut WktWriter::new(&mut wkt_data)).is_ok());

        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), wkt);
    }

    #[test]
    fn multipolygon_geom() {
        let wkt = "MULTIPOLYGON(((0 0,0 1,1 1,1 0,0 0)))";
        let ggeom = GGeometry::new_from_wkt(wkt).unwrap();

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_geom(&ggeom, &mut WktWriter::new(&mut wkt_data)).is_ok());

        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), wkt);
    }

    #[test]
    fn geometry_collection_geom() {
        let wkt = "GEOMETRYCOLLECTION(POINT(1 1),LINESTRING(1 1,2 2))";
        let ggeom = GGeometry::new_from_wkt(wkt).unwrap();

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_geom(&ggeom, &mut WktWriter::new(&mut wkt_data)).is_ok());

        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), wkt);
    }
}
