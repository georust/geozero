use std::io::Read;

use byteorder::{BigEndian, ReadBytesExt};

use crate::geometry_processor::bounded_vec;
use crate::shp::shp_reader::validate_count;
use crate::shp::{Error, header};

const INDEX_RECORD_SIZE: usize = 2 * size_of::<i32>();

pub(crate) struct ShapeIndex {
    #[allow(dead_code)]
    pub offset: i32,
    #[allow(dead_code)]
    pub record_size: i32,
}

/// Read the content of a .shx file
pub(crate) fn read_index_file<T: Read>(mut source: T) -> Result<Vec<ShapeIndex>, Error> {
    let header = header::Header::read_from(&mut source)?;

    // `file_length` is an untrusted `i32` in 16-bit words. A negative value cast
    // `as usize` sign-extends to ~1.8e19 and the `Vec` reservation below panics
    // (`capacity overflow`) or requests a multi-GB allocation from a tiny `.shx`.
    let file_length = validate_count(header.file_length)?;
    let file_length_bytes = file_length
        .checked_mul(2)
        .ok_or(Error::InvalidShapeRecordSize)?;
    if file_length_bytes < header::HEADER_SIZE as usize {
        // A truncated/empty index yields no records rather than an underflowing
        // (wrapping) shape count.
        return Ok(Vec::new());
    }
    let num_shapes = (file_length_bytes - header::HEADER_SIZE as usize) / INDEX_RECORD_SIZE;

    let mut shapes_index = bounded_vec::<ShapeIndex>(num_shapes)?;
    for _ in 0..num_shapes {
        let offset = source.read_i32::<BigEndian>()?;
        let record_size = source.read_i32::<BigEndian>()?;
        shapes_index.push(ShapeIndex {
            offset,
            record_size,
        });
    }
    Ok(shapes_index)
}
