use crate::GeomProcessor;
use crate::shp::{Error, ShapeType};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::Read;
use std::mem::size_of;

/// Value inferior to this are considered as NO_DATA
pub const NO_DATA: f64 = -10e38;

pub(crate) fn is_no_data(val: f64) -> bool {
    val <= NO_DATA
}

/// Header of a shape record, present before any shape record
pub(crate) struct RecordHeader {
    #[allow(dead_code)]
    pub record_number: i32,
    pub record_size: i32,
}

impl RecordHeader {
    pub(crate) const SIZE: usize = 2 * size_of::<i32>();

    pub fn read_from<T: Read>(source: &mut T) -> Result<RecordHeader, Error> {
        let record_number = source.read_i32::<BigEndian>()?;
        let record_size = source.read_i32::<BigEndian>()?;
        Ok(RecordHeader {
            record_number,
            record_size,
        })
    }
}

/// Read and process one shape record
pub(crate) fn read_shape<'a, P: GeomProcessor + 'a, T: Read>(
    processor: &'a mut P,
    mut source: &mut T,
) -> Result<RecordHeader, Error> {
    let hdr = RecordHeader::read_from(&mut source)?;
    let record_size = hdr.record_size * 2;
    read_shape_rec(processor, &mut source, record_size as usize)?;
    Ok(hdr)
}

fn read_shape_rec<P: GeomProcessor, T: Read>(
    processor: &mut P,
    mut source: &mut T,
    record_size: usize,
) -> Result<(), Error> {
    let shape_type = ShapeType::read_from(&mut source)?;
    let record_size = record_size - size_of::<i32>();
    match shape_type {
        ShapeType::Point => read_point(processor, &mut source, record_size, shape_type)?,
        ShapeType::PointM => read_point(processor, &mut source, record_size, shape_type)?,
        ShapeType::PointZ => read_point(processor, &mut source, record_size, shape_type)?,
        ShapeType::Multipoint => {
            read_multipoint(processor, &mut source, record_size, ShapeType::Point)?
        }
        ShapeType::MultipointM => {
            read_multipoint(processor, &mut source, record_size, ShapeType::PointM)?
        }
        ShapeType::MultipointZ => {
            read_multipoint(processor, &mut source, record_size, ShapeType::PointZ)?
        }
        ShapeType::Polyline => read_polyline(processor, &mut source, record_size, false)?,
        ShapeType::PolylineM => read_polyline(processor, &mut source, record_size, false)?,
        ShapeType::PolylineZ => read_polyline(processor, &mut source, record_size, true)?,
        ShapeType::Polygon => read_polygon(processor, &mut source, record_size, false)?,
        ShapeType::PolygonM => read_polygon(processor, &mut source, record_size, false)?,
        ShapeType::PolygonZ => read_polygon(processor, &mut source, record_size, true)?,
        ShapeType::Multipatch => {
            read_multipatch_shape_content(processor, &mut source, record_size)?
        }
        ShapeType::NullShape => {}
    };
    Ok(())
}

fn read_point<P: GeomProcessor, T: Read>(
    processor: &mut P,
    source: &mut T,
    record_size: usize,
    point_type: ShapeType,
) -> Result<(), Error> {
    let has_z = point_type == ShapeType::PointZ;
    let has_m = point_type == ShapeType::PointM
        || point_type == ShapeType::PointZ && record_size == 4 * size_of::<f64>();
    let mut dims = 2;
    if has_z {
        dims += 1;
    }
    if has_m {
        dims += 1;
    }
    if record_size != dims * size_of::<f64>() {
        return Err(Error::InvalidShapeRecordSize);
    }
    let x = source.read_f64::<LittleEndian>()?;
    let y = source.read_f64::<LittleEndian>()?;
    let z = if has_z {
        Some(source.read_f64::<LittleEndian>()?)
    } else {
        None
    };
    let m = if has_m {
        Some(source.read_f64::<LittleEndian>()?)
    } else {
        None
    };

    processor.point_begin(0)?;
    if processor.multi_dim() {
        let dimensions = processor.dimensions();
        let z = if dimensions.z { z } else { None };
        let m = if dimensions.m { m } else { None };
        processor.coordinate(x, y, z, m, None, None, 0)?;
    } else {
        processor.xy(x, y, 0)?;
    }
    processor.point_end(0)?;
    Ok(())
}

fn read_multipoint<P: GeomProcessor, T: Read>(
    processor: &mut P,
    source: &mut T,
    record_size: usize,
    point_type: ShapeType,
) -> Result<(), Error> {
    let _bbox = read_bbox(source, 2)?;
    let num_points = source.read_i32::<LittleEndian>()? as usize;

    let mut size = 4 * size_of::<f64>() // BBOX
    + size_of::<i32>() // num points
    + size_of::<f64>() * 2 * num_points;
    let has_z = point_type == ShapeType::PointZ;
    if has_z {
        size += multipart_dim_value_size(num_points);
    }
    let has_m = record_size == size + multipart_dim_value_size(num_points);
    if has_m {
        size += multipart_dim_value_size(num_points);
    }

    if record_size != size {
        return Err(Error::InvalidShapeRecordSize);
    }

    let coords = read_xy(source, num_points)?;
    let z_values = if has_z {
        read_dim_values(source, num_points)?
    } else {
        Vec::new()
    };
    let m_values = if has_m {
        read_dim_values(source, num_points)?
    } else {
        Vec::new()
    };

    let multi_dim = processor.multi_dim();
    let dimensions = processor.dimensions();
    let get_z = dimensions.z && !z_values.is_empty();
    let get_m = dimensions.m && !m_values.is_empty();

    processor.multipoint_begin(num_points, 0)?;
    for idx in 0..num_points {
        let coord = &coords[idx];
        if !multi_dim {
            processor.xy(coord.x, coord.y, idx)?;
        } else {
            let z = if get_z { Some(z_values[idx]) } else { None };
            let m = if get_m { Some(m_values[idx]) } else { None };
            processor.coordinate(coord.x, coord.y, z, m, None, None, idx)?;
        }
    }
    processor.multipoint_end(0)?;
    Ok(())
}

fn read_polyline<P: GeomProcessor, T: Read>(
    processor: &mut P,
    source: &mut T,
    record_size: usize,
    has_z: bool,
) -> Result<(), Error> {
    let multipart = MultiPartShape::read(source, record_size, has_z)?;
    multipart.process(processor, false)?;
    Ok(())
}

fn read_polygon<P: GeomProcessor, T: Read>(
    processor: &mut P,
    source: &mut T,
    record_size: usize,
    has_z: bool,
) -> Result<(), Error> {
    let multipart = MultiPartShape::read(source, record_size, has_z)?;
    multipart.process(processor, true)?;
    Ok(())
}

fn read_multipatch_shape_content<P: GeomProcessor, T: Read>(
    _processor: &mut P,
    source: &mut T,
    record_size: usize,
) -> Result<(), Error> {
    // TODO
    let mut buffer = vec![0; record_size];
    source.read_exact(&mut buffer)?;
    Ok(())
}

// --- multipart line reader ---

struct MultiPartShape {
    parts_index: Vec<usize>,
    coords: Vec<Coord>,
    z_values: Vec<f64>,
    m_values: Vec<f64>,
}

struct Coord {
    x: f64,
    y: f64,
}

impl MultiPartShape {
    fn read<R: Read>(
        source: &mut R,
        record_size: usize,
        has_z: bool,
    ) -> Result<MultiPartShape, Error> {
        let _bbox = read_bbox(source, 2)?;
        let num_parts = source.read_i32::<LittleEndian>()? as usize;
        let num_points = source.read_i32::<LittleEndian>()? as usize;
        let mut rec_size = multipart_record_size(num_points, num_parts);
        if has_z {
            rec_size += multipart_dim_value_size(num_points);
        }
        let has_m = record_size == rec_size + multipart_dim_value_size(num_points);
        if record_size != rec_size && !has_m {
            return Err(Error::InvalidShapeRecordSize);
        }

        let mut parts_index = Vec::with_capacity(num_parts + 1);
        for _ in 0..num_parts {
            parts_index.push(source.read_i32::<LittleEndian>()? as usize);
        }
        parts_index.push(num_points); // add last index to simplify iteration

        let mut multipart = MultiPartShape {
            parts_index,
            coords: Vec::new(),
            z_values: Vec::new(),
            m_values: Vec::new(),
        };
        multipart.coords = read_xy(source, num_points)?;
        if has_z {
            multipart.z_values = read_dim_values(source, num_points)?;
        }
        if has_m {
            multipart.m_values = read_dim_values(source, num_points)?;
        }

        Ok(multipart)
    }

    fn num_parts(&self) -> usize {
        self.parts_index.len() - 1
    }

    fn detect_polys(&self) -> Vec<usize> {
        let mut polys = Vec::with_capacity(self.parts_index.len());
        for idx in 0..self.parts_index.len() - 1 {
            let (start_index, end_index) = (self.parts_index[idx], self.parts_index[idx + 1]);
            if idx == 0
                || ring_type_from_points_ordering(&self.coords[start_index..end_index])
                    == RingType::OuterRing
            {
                polys.push(idx);
            }
        }
        polys.push(self.parts_index.len() - 1);
        polys
    }

    fn process<P: GeomProcessor>(&self, processor: &mut P, as_poly: bool) -> Result<(), Error> {
        let tagged = false;
        let dimensions = processor.dimensions();
        let multi_dim = dimensions.z || dimensions.m;
        let get_z = dimensions.z && !self.z_values.is_empty();
        let get_m = dimensions.m && !self.m_values.is_empty();

        let geom_parts_indices = if as_poly {
            self.detect_polys()
        } else {
            vec![0, self.parts_index.len() - 1]
        };

        if as_poly {
            processor.multipolygon_begin(geom_parts_indices.len() - 1, 0)?;
        } else {
            processor.multilinestring_begin(self.num_parts(), 0)?;
        }
        for (geom_idx, geom_start_end) in geom_parts_indices.windows(2).enumerate() {
            let (geom_start, geom_end) = (geom_start_end[0], geom_start_end[1]);
            let num_rings = geom_end - geom_start;
            if as_poly {
                processor.polygon_begin(tagged, num_rings, geom_idx)?;
            }
            for (ring_idx, start_end) in self.parts_index[geom_start..=geom_end]
                .windows(2)
                .enumerate()
            {
                let (start_index, end_index) = (start_end[0], start_end[1]);
                let num_points_in_part = end_index - start_index;
                processor.linestring_begin(tagged, num_points_in_part, ring_idx)?;
                for ofs in start_index..end_index {
                    let coord_idx = ofs - start_index;
                    let coord = &self.coords[ofs];
                    if !multi_dim {
                        processor.xy(coord.x, coord.y, coord_idx)?;
                    } else {
                        let z = if get_z {
                            Some(self.z_values[ofs])
                        } else {
                            None
                        };
                        let m = if get_m {
                            Some(self.m_values[ofs])
                        } else {
                            None
                        };
                        processor.coordinate(coord.x, coord.y, z, m, None, None, coord_idx)?;
                    }
                }
                processor.linestring_end(tagged, ring_idx)?;
            }
            if as_poly {
                processor.polygon_end(tagged, geom_idx)?;
            }
        }
        if as_poly {
            processor.multipolygon_end(0)?;
        } else {
            processor.multilinestring_end(0)?;
        }

        Ok(())
    }
}

fn multipart_record_size(num_points: usize, num_parts: usize) -> usize {
    let mut size = 0usize;
    size += 4 * size_of::<f64>(); // BBOX
    size += size_of::<i32>(); // num parts
    size += size_of::<i32>(); // num points
    size += size_of::<i32>() * num_parts;
    size += size_of::<f64>() * 2 * num_points;
    size
}

fn multipart_dim_value_size(num_points: usize) -> usize {
    2 * size_of::<f64>() // range
     + num_points * size_of::<f64>() // values
}

fn read_bbox<R: Read>(source: &mut R, dims: usize) -> Result<Vec<f64>, Error> {
    let mut bbox = Vec::with_capacity(2 * dims);
    for _ in 0..bbox.capacity() {
        bbox.push(source.read_f64::<LittleEndian>()?);
    }
    Ok(bbox)
}

fn read_xy<R: Read>(source: &mut R, num_points: usize) -> Result<Vec<Coord>, Error> {
    let mut coords = Vec::with_capacity(num_points);
    for _ in 0..num_points {
        let x = source.read_f64::<LittleEndian>()?;
        let y = source.read_f64::<LittleEndian>()?;
        coords.push(Coord { x, y });
    }
    Ok(coords)
}

fn read_dim_values<R: Read>(source: &mut R, num_points: usize) -> Result<Vec<f64>, Error> {
    let _range = read_bbox(source, 1)?;
    let mut values = Vec::with_capacity(num_points);
    for _ in 0..num_points {
        values.push(source.read_f64::<LittleEndian>()?);
    }
    Ok(values)
}

#[derive(Eq, PartialEq, Debug)]
enum RingType {
    OuterRing,
    InnerRing,
}

/// Given the points, check if they represent an outer ring of a polygon
///
/// As per ESRI's Shapefile 1998 whitepaper:
/// `
/// The order of vertices or orientation for a ring indicates which side of the ring
/// is the interior of the polygon.
/// The neighborhood to the right of an observer walking along
/// the ring in vertex order is the neighborhood inside the polygon.
/// Vertices of rings defining holes in polygons are in a counterclockwise direction.
/// Vertices for a single, ringed polygon are, therefore, always in clockwise order.
/// `
///
/// Inner Rings defines holes -> points are in counterclockwise order
/// Outer Rings' points are un clockwise order
///
/// https://stackoverflow.com/questions/1165647/how-to-determine-if-a-list-of-polygon-points-are-in-clockwise-order/1180256#1180256
fn ring_type_from_points_ordering(points: &[Coord]) -> RingType {
    let area = points
        .windows(2)
        .map(|pts| (pts[1].x - pts[0].x) * (pts[1].y + pts[0].y))
        .sum::<f64>()
        / 2.0f64;

    if area < 0.0 {
        RingType::InnerRing
    } else {
        RingType::OuterRing
    }
}
