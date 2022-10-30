use crate::error::Result;
use crate::{GeomProcessor, GeozeroGeometry};
use arrow2::array::{Array, ListArray, Offset, PrimitiveArray, StructArray};
use arrow2::datatypes::DataType;

/// A bare StructArray is a collection of points
impl GeozeroGeometry for StructArray {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        process_points(self, processor)
    }
}

impl GeozeroGeometry for ListArray<i32> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        delegate_by_extension_type(self, processor)
    }
}

impl GeozeroGeometry for ListArray<i64> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        delegate_by_extension_type(self, processor)
    }
}

fn delegate_by_extension_type<T: Offset>(
    array: &ListArray<T>,
    processor: &mut impl GeomProcessor,
) -> Result<()> {
    match array.data_type() {
        DataType::Extension(name, _, _) if name == "geoarrow.multipoint" => {
            process_multi_point(array, processor)
        }
        DataType::Extension(name, _, _) if name == "geoarrow.linestring" => {
            process_linestring(array, processor)
        }
        DataType::Extension(name, _, _) if name == "geoarrow.multilinestring" => {
            process_multi_linestring(array, processor)
        }
        DataType::Extension(name, _, _) if name == "geoarrow.polygon" => {
            process_polygon(array, processor)
        }
        DataType::Extension(name, _, _) if name == "geoarrow.multipolygon" => {
            process_multi_polygon(array, processor)
        }
        _ => panic!("Unsupported data type. Should be a geoarrow extension type."),
    }
}

pub fn process_points(array: &StructArray, processor: &mut impl GeomProcessor) -> Result<()> {
    let struct_array_values = array.values();
    let x_arrow_array = &struct_array_values[0];
    let y_arrow_array = &struct_array_values[1];

    let x_array_values = x_arrow_array
        .as_any()
        .downcast_ref::<PrimitiveArray<f64>>()
        .unwrap();
    let y_array_values = y_arrow_array
        .as_any()
        .downcast_ref::<PrimitiveArray<f64>>()
        .unwrap();

    let array_len = array.len();
    processor.geometrycollection_begin(array_len, 0)?;

    for idx in 0..array_len {
        let x = x_array_values.value(idx);
        let y = y_array_values.value(idx);

        processor.point_begin(idx)?;
        processor.xy(x, y, idx)?;
        processor.point_end(idx)?;
    }

    processor.geometrycollection_end(array_len - 1)?;

    Ok(())
}

fn process_multi_point<T: Offset>(
    array: &ListArray<T>,
    processor: &mut impl GeomProcessor,
) -> Result<()> {
    let offsets = array.offsets();
    let inner_struct_array = array
        .values()
        .as_any()
        .downcast_ref::<StructArray>()
        .unwrap();

    let x_array_values = inner_struct_array.values()[0]
        .as_any()
        .downcast_ref::<PrimitiveArray<f64>>()
        .unwrap();
    let y_array_values = inner_struct_array.values()[1]
        .as_any()
        .downcast_ref::<PrimitiveArray<f64>>()
        .unwrap();

    let array_len = array.len();
    processor.geometrycollection_begin(array_len, 0)?;

    for geom_idx in 0..array_len {
        let begin_offset = offsets[geom_idx];
        let end_offset = offsets[geom_idx + 1];
        let n_pts = end_offset - begin_offset;
        processor.multipoint_begin(n_pts.to_usize(), geom_idx)?;

        for value_idx in begin_offset.to_usize()..end_offset.to_usize() {
            let x = x_array_values.value(value_idx);
            let y = y_array_values.value(value_idx);
            processor.xy(x, y, value_idx - begin_offset.to_usize())?;
        }

        processor.multipoint_end(geom_idx)?;
    }

    processor.geometrycollection_end(array_len - 1)?;

    Ok(())
}

fn process_linestring<T: Offset>(
    array: &ListArray<T>,
    processor: &mut impl GeomProcessor,
) -> Result<()> {
    todo!()
}

fn process_multi_linestring<T: Offset>(
    array: &ListArray<T>,
    processor: &mut impl GeomProcessor,
) -> Result<()> {
    todo!()
}

fn process_polygon<T: Offset>(
    array: &ListArray<T>,
    processor: &mut impl GeomProcessor,
) -> Result<()> {
    todo!()
}

fn process_multi_polygon<T: Offset>(
    array: &ListArray<T>,
    processor: &mut impl GeomProcessor,
) -> Result<()> {
    todo!()
}
