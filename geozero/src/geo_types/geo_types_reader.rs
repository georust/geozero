use crate::error::Result;
use crate::{GeomProcessor, GeozeroGeometry};
use geo_types::*;

impl GeozeroGeometry for geo_types::Geometry<f64> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        process_geom(self, processor)
    }
}

/// Process geo-types geometry.
pub fn process_geom<P: GeomProcessor>(geom: &Geometry<f64>, processor: &mut P) -> Result<()> {
    process_geom_n(geom, 0, processor)
}

fn process_geom_n<P: GeomProcessor>(
    geom: &Geometry<f64>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    match geom {
        Geometry::Point(ref geom) => {
            processor.point_begin(idx)?;
            process_coord(&geom.0, 0, processor)?;
            processor.point_end(idx)?;
        }
        Geometry::Line(geom) => {
            processor.linestring_begin(true, 2, idx)?;
            process_coord(&geom.start, 0, processor)?;
            process_coord(&geom.end, 1, processor)?;
            processor.linestring_end(true, idx)?;
        }
        Geometry::LineString(ref geom) => {
            process_linestring(geom, true, idx, processor)?;
        }
        Geometry::Polygon(ref geom) => {
            process_polygon(geom, true, idx, processor)?;
        }
        Geometry::MultiPoint(ref geom) => {
            processor.multipoint_begin(geom.0.len(), idx)?;
            for (i, pt) in geom.0.iter().enumerate() {
                process_coord(&pt.0, i, processor)?;
            }
            processor.multipoint_end(idx)?;
        }
        Geometry::MultiLineString(ref geom) => {
            processor.multilinestring_begin(geom.0.len(), idx)?;
            for (i, line) in geom.0.iter().enumerate() {
                process_linestring(line, false, i, processor)?;
            }
            processor.multilinestring_end(idx)?;
        }
        Geometry::MultiPolygon(ref geom) => {
            processor.multipolygon_begin(geom.0.len(), idx)?;
            for (i, poly) in geom.0.iter().enumerate() {
                process_polygon(poly, false, i, processor)?;
            }
            processor.multipolygon_end(idx)?;
        }
        Geometry::GeometryCollection(ref geom) => {
            processor.geometrycollection_begin(geom.0.len(), idx)?;
            for (i, g) in geom.0.iter().enumerate() {
                process_geom_n(g, i, processor)?;
            }
            processor.geometrycollection_end(idx)?;
        }
        Geometry::Rect(geom) => {
            process_polygon(&geom.to_polygon(), true, idx, processor)?;
        }
        Geometry::Triangle(geom) => {
            process_polygon(&geom.to_polygon(), true, idx, processor)?;
        }
    }
    Ok(())
}

fn process_coord<P: GeomProcessor>(
    coord: &Coordinate<f64>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    if processor.multi_dim() {
        processor.coordinate(coord.x, coord.y, None, None, None, None, idx)?;
    } else {
        processor.xy(coord.x, coord.y, idx)?;
    }
    Ok(())
}

fn process_linestring<P: GeomProcessor>(
    geom: &LineString<f64>,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let multi = processor.multi_dim();
    processor.linestring_begin(tagged, geom.0.len(), idx)?;
    for (i, coord) in geom.0.iter().enumerate() {
        if multi {
            processor.coordinate(coord.x, coord.y, None, None, None, None, i)?;
        } else {
            processor.xy(coord.x, coord.y, i)?;
        }
    }
    processor.linestring_end(tagged, idx)
}

fn process_polygon<P: GeomProcessor>(
    geom: &Polygon<f64>,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let interiors = geom.interiors();
    processor.polygon_begin(tagged, interiors.len() + 1, idx)?;
    // Exterior ring
    process_linestring(geom.exterior(), false, 0, processor)?;
    // Interior rings
    for (i, ring) in interiors.iter().enumerate() {
        process_linestring(ring, false, i + 1, processor)?;
    }
    processor.polygon_end(tagged, idx)
}

#[cfg(test)]
#[cfg(feature = "with-wkt")]
mod test {
    use super::*;
    use crate::wkt::WktWriter;
    use crate::ToWkt;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    fn point() {
        let wkt = "POINT(1 1)";
        let geo = Geometry::try_from(wkt::Wkt::from_str(wkt).unwrap()).unwrap();

        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(process_geom(&geo, &mut WktWriter::new(&mut wkt_data)).is_ok());

        assert_eq!(std::str::from_utf8(&wkt_data).unwrap(), wkt);
    }

    #[test]
    fn multipoint() {
        let wkt = "MULTIPOINT(1 1,2 2)";
        let geo =
            Geometry::try_from(wkt::Wkt::from_str("MULTIPOINT((1 1),(2 2))").unwrap()).unwrap();
        assert_eq!(geo.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn line() {
        let wkt = "LINESTRING(1 1,2 2)";
        let geo = Geometry::try_from(wkt::Wkt::from_str(wkt).unwrap()).unwrap();
        assert_eq!(geo.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn multiline() {
        let wkt = "MULTILINESTRING((1 1,2 2),(3 3,4 4))";
        let geo = Geometry::try_from(wkt::Wkt::from_str(wkt).unwrap()).unwrap();
        assert_eq!(geo.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn polygon() {
        let wkt = "POLYGON((0 0,0 3,3 3,3 0,0 0),(0.2 0.2,0.2 2,2 2,2 0.2,0.2 0.2))";
        let geo = Geometry::try_from(wkt::Wkt::from_str(wkt).unwrap()).unwrap();
        assert_eq!(geo.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn multipolygon() {
        let wkt = "MULTIPOLYGON(((0 0,0 1,1 1,1 0,0 0)))";
        let geo = Geometry::try_from(wkt::Wkt::from_str(wkt).unwrap()).unwrap();
        assert_eq!(geo.to_wkt().unwrap(), wkt);
    }

    #[test]
    fn geometry_collection() {
        let wkt = "GEOMETRYCOLLECTION(POINT(1 1),LINESTRING(1 1,2 2))";
        let geo = Geometry::try_from(wkt::Wkt::from_str(wkt).unwrap()).unwrap();
        assert_eq!(geo.to_wkt().unwrap(), wkt);
    }
}
