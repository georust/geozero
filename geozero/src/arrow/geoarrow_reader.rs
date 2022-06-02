use crate::error::Result;
use crate::wkb::wkb_reader::{process_wkb_geom_n, read_wkb_header};
use crate::{GeomProcessor, GeozeroGeometry};
use arrow2::array::BinaryArray;

/// GeoArrow WKB reader
// TODO: BinaryArray is generic over the type used for offsets: i32 and i64. This should not be
// hardcoded as i32
#[derive(Debug)]
pub struct GeoArrowWkb(pub BinaryArray<i32>);

impl GeozeroGeometry for GeoArrowWkb {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        // TODO: how does this fn know which index to pick?
        // Or is this fn intended to work on a single geometry?
        process_geoarrow_wkb_geom(&self.0, processor)
    }
}

pub fn process_geoarrow_wkb_geom(
    array: &BinaryArray<i32>,
    processor: &mut impl GeomProcessor,
) -> Result<()> {
    let array_len = array.len();
    processor.geometrycollection_begin(array_len, 0)?;

    for i in 0..array_len {
        let raw = &mut array.value(i);
        let info = read_wkb_header(raw)?;
        process_wkb_geom_n(raw, &info, read_wkb_header, i, processor)?;
    }

    processor.geometrycollection_end(array_len - 1)?;

    Ok(())
}
