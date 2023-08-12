use crate::error::Result;
use crate::wkb::wkb_reader::{process_wkb_geom_n, read_wkb_header, read_wkb_nested_header};
use crate::{GeomProcessor, GeozeroGeometry};
use arrow2::array::BinaryArray;
use arrow2::types::Offset;

impl GeozeroGeometry for BinaryArray<i32> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        process_geoarrow_wkb_geom(self, processor)
    }
}

impl GeozeroGeometry for BinaryArray<i64> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        process_geoarrow_wkb_geom(self, processor)
    }
}

pub fn process_geoarrow_wkb_geom<T: Offset>(
    array: &BinaryArray<T>,
    processor: &mut impl GeomProcessor,
) -> Result<()> {
    let array_len = array.len();
    processor.geometrycollection_begin(array_len, 0)?;

    for i in 0..array_len {
        let raw = &mut array.value(i);
        let info = read_wkb_header(raw)?;
        process_wkb_geom_n(raw, &info, read_wkb_nested_header, i, processor)?;
    }

    processor.geometrycollection_end(array_len - 1)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::wkt::conversion::ToWkt;
    use arrow2::io::ipc::read;
    use std::fs::File;

    #[test]
    fn multipoly_file() -> arrow2::error::Result<()> {
        let mut file = File::open("tests/data/countries.arrow")?;
        let metadata = read::read_file_metadata(&mut file)?;
        let mut reader = read::FileReader::new(file, metadata, None, None);

        let columns = reader.next().unwrap()?;

        let array = &columns.arrays()[2];
        let wkbarr = array.as_any().downcast_ref::<BinaryArray<i32>>().unwrap();
        let wkt = wkbarr.to_wkt().unwrap();
        assert_eq!(
            &wkt[0..100],
            "GEOMETRYCOLLECTION(MULTIPOLYGON(((-59.572095 -80.040179,-59.865849 -80.549657,-60.159656 -81.000327,"
        );
        assert_eq!(
            &wkt[wkt.len()-100..],
            "-51.5,-58.55 -51.1,-57.75 -51.55,-58.05 -51.9,-59.4 -52.2,-59.85 -51.85,-60.7 -52.3,-61.2 -51.85))))"
        );
        Ok(())
    }
}
