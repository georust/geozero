use crate::{header, Error};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Read;

const INDEX_RECORD_SIZE: usize = 2 * std::mem::size_of::<i32>();

pub(crate) struct ShapeIndex {
    pub offset: i32,
    pub record_size: i32,
}

/// Read the content of a .shx file
pub(crate) fn read_index_file<T: Read>(mut source: T) -> Result<Vec<ShapeIndex>, Error> {
    let header = header::Header::read_from(&mut source)?;

    let num_shapes = ((header.file_length * 2) - header::HEADER_SIZE) / INDEX_RECORD_SIZE as i32;
    let mut shapes_index = Vec::<ShapeIndex>::with_capacity(num_shapes as usize);
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
