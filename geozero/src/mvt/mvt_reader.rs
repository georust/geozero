use crate::error::{GeozeroError, Result};
use crate::mvt::vector_tile::{tile, tile::GeomType};
use crate::{ColumnValue, FeatureProcessor, GeomProcessor, GeozeroDatasource, GeozeroGeometry};

use super::mvt_commands::{Command, CommandInteger, ParameterInteger};

impl GeozeroDatasource for tile::Layer {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        process(&self, processor)
    }
}

/// Process MVT layer.
pub fn process(layer: &tile::Layer, processor: &mut impl FeatureProcessor) -> Result<()> {
    processor.dataset_begin(Some(&layer.name))?;
    for (idx, feature) in layer.features.iter().enumerate() {
        processor.feature_begin(idx as u64)?;

        process_properties(layer, feature, processor)?;

        processor.geometry_begin()?;
        process_geom(feature, processor)?;
        processor.geometry_end()?;

        processor.feature_end(idx as u64)?;
    }
    processor.dataset_end()?;
    Ok(())
}

fn process_properties(
    layer: &tile::Layer,
    feature: &tile::Feature,
    processor: &mut impl FeatureProcessor,
) -> Result<()> {
    processor.properties_begin()?;
    for i in 0..feature.tags.len() / 2 {
        let key_idx = feature.tags[i * 2];
        let value_idx = feature.tags[i * 2 + 1];
        let key = layer
            .keys
            .get(key_idx as usize)
            .ok_or_else(|| GeozeroError::Feature(format!("invalid key index {}", key_idx)))?;
        let value = layer
            .values
            .get(value_idx as usize)
            .ok_or_else(|| GeozeroError::Feature(format!("invalid value index {}", value_idx)))?;

        if let Some(ref v) = value.string_value {
            processor.property(i, key, &ColumnValue::String(v))?;
        } else if let Some(v) = value.float_value {
            processor.property(i, key, &ColumnValue::Float(v))?;
        } else if let Some(v) = value.double_value {
            processor.property(i, key, &ColumnValue::Double(v))?;
        } else if let Some(v) = value.int_value {
            processor.property(i, key, &ColumnValue::Long(v))?;
        } else if let Some(v) = value.uint_value {
            processor.property(i, key, &ColumnValue::ULong(v))?;
        } else if let Some(v) = value.sint_value {
            processor.property(i, key, &ColumnValue::Long(v))?;
        } else if let Some(v) = value.bool_value {
            processor.property(i, key, &ColumnValue::Bool(v))?;
        } else {
            return Err(GeozeroError::Property(format!(
                "unsupported value type for key {}",
                key
            )));
        }
    }
    processor.properties_end()?;
    Ok(())
}

impl GeozeroGeometry for tile::Feature {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        process_geom(self, processor)
    }
}

/// Process MVT geometry.
pub fn process_geom<P: GeomProcessor>(geom: &tile::Feature, processor: &mut P) -> Result<()> {
    process_geom_n(geom, 0, processor)
}

fn process_geom_n<P: GeomProcessor>(
    geom: &tile::Feature,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let mut cursor: [i32; 2] = [0, 0];
    match geom.r#type {
        Some(r#type) if r#type == GeomType::Point as i32 => {
            process_point(&mut cursor, &geom.geometry, idx, processor);
        }
        Some(r#type) if r#type == GeomType::Linestring as i32 => {
            process_linestrings(&mut cursor, geom, idx, processor)?;
        }
        Some(r#type) if r#type == GeomType::Polygon as i32 => {
            process_polygons(&mut cursor, geom, idx, processor)?;
        }
        _ => {}
    }
    Ok(())
}

fn process_coord<P: GeomProcessor>(
    cursor: &mut [i32; 2],
    coord: &[u32],
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    cursor[0] += ParameterInteger(coord[0]).value();
    cursor[1] += ParameterInteger(coord[1]).value();
    if processor.multi_dim() {
        processor.coordinate(
            cursor[0] as f64,
            cursor[1] as f64,
            None,
            None,
            None,
            None,
            idx,
        )?;
    } else {
        processor.xy(cursor[0] as f64, cursor[1] as f64, idx)?;
    }
    Ok(())
}

fn process_point<P: GeomProcessor>(
    cursor: &mut [i32; 2],
    geom: &[u32],
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let command = CommandInteger(geom[0]);
    let count = command.count() as usize;
    if count == 1 {
        processor.point_begin(idx)?;
        process_coord(cursor, &geom[1..3], 0, processor)?;
        processor.point_end(idx)?;
    } else {
        processor.multipoint_begin(count, idx)?;
        for i in 0..count {
            process_coord(cursor, &geom[1 + i * 2..3 + i * 2], i, processor)?;
        }
        processor.multipoint_end(idx)?;
    }
    Ok(())
}

fn process_linestring<P: GeomProcessor>(
    cursor: &mut [i32; 2],
    geom: &[u32],
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    if geom[0] != CommandInteger::from(Command::MoveTo, 1) {
        return Err(GeozeroError::GeometryFormat);
    }
    let lineto = CommandInteger(geom[3]);
    if lineto.id() != Command::LineTo as u32 {
        return Err(GeozeroError::GeometryFormat);
    }
    processor.linestring_begin(tagged, 1 + lineto.count() as usize, idx);
    process_coord(cursor, &geom[1..3], 0, processor)?;
    for i in 0..lineto.count() as usize {
        process_coord(cursor, &geom[4 + i * 2..6 + i * 2], i + 1, processor)?;
    }
    processor.linestring_end(tagged, idx);
    Ok(())
}

fn process_linestrings<P: GeomProcessor>(
    cursor: &mut [i32; 2],
    geom: &tile::Feature,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let mut line_string_slices: Vec<&[u32]> = vec![];
    let mut geom: &[u32] = &geom.geometry;

    while geom.len() > 0 {
        let lineto = CommandInteger(geom[3]);
        let slice_size = 4 + lineto.count() as usize * 2;
        let (slice, rest) = geom.split_at(slice_size);
        line_string_slices.push(slice);
        geom = rest;
    }

    if line_string_slices.len() > 1 {
        processor.multilinestring_begin(line_string_slices.len(), idx);
        for i in 0..line_string_slices.len() {
            process_linestring(cursor, line_string_slices[i], false, i, processor)?;
        }
        processor.multilinestring_end(idx);
    } else {
        process_linestring(cursor, line_string_slices[0], true, idx, processor)?;
    }

    Ok(())
}

fn process_polygon<P: GeomProcessor>(
    cursor: &mut [i32; 2],
    rings: &[&[u32]],
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.polygon_begin(tagged, rings.len(), idx);

    for (i, ring) in rings.iter().enumerate() {
        if ring[0] != CommandInteger::from(Command::MoveTo, 1) {
            return Err(GeozeroError::GeometryFormat);
        }
        if *ring.last().unwrap() != CommandInteger::from(Command::ClosePath, 1) {
            return Err(GeozeroError::GeometryFormat);
        }
        let lineto = CommandInteger(ring[3]);
        if lineto.id() != Command::LineTo as u32 {
            return Err(GeozeroError::GeometryFormat);
        }
        processor.linestring_begin(false, 1 + lineto.count() as usize, i);
        let mut start_cursor = cursor.clone();
        process_coord(cursor, &ring[1..3], 0, processor)?;
        for i in 0..lineto.count() as usize {
            process_coord(cursor, &ring[4 + i * 2..6 + i * 2], i + 1, processor)?;
        }
        process_coord(
            &mut start_cursor,
            &ring[1..3],
            1 + lineto.count() as usize,
            processor,
        )?;
        processor.linestring_end(false, i);
    }

    processor.polygon_end(tagged, idx);

    Ok(())
}

fn process_polygons<P: GeomProcessor>(
    cursor: &mut [i32; 2],
    geom: &tile::Feature,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let mut polygon_slices: Vec<Vec<&[u32]>> = vec![];
    let mut geom: &[u32] = &geom.geometry;

    while geom.len() > 0 {
        let lineto = CommandInteger(geom[3]);
        let slice_size = 4 + lineto.count() as usize * 2 + 1;
        let (slice, rest) = geom.split_at(slice_size);
        let positive_area = is_area_positive(
            cursor.clone(),
            &slice[1..3],
            &slice[4..4 + lineto.count() as usize * 2],
        );
        if positive_area {
            // new polygon with exterior ring
            polygon_slices.push(vec![slice]);
        } else if let Some(last_slice) = polygon_slices.last_mut() {
            // add interior ring to previous polygon
            last_slice.push(slice);
        } else {
            return Err(GeozeroError::GeometryFormat);
        }
        geom = rest;
    }

    if polygon_slices.len() > 1 {
        processor.multipolygon_begin(polygon_slices.len(), idx);
        for i in 0..polygon_slices.len() {
            process_polygon(cursor, &polygon_slices[i], false, i, processor)?;
        }
        processor.multipolygon_end(idx);
    } else {
        process_polygon(cursor, &polygon_slices[0], true, idx, processor)?;
    }

    Ok(())
}

// using surveyor's formula
fn is_area_positive(mut cursor: [i32; 2], first: &[u32], rest: &[u32]) -> bool {
    let nb = 1 + rest.len() / 2;
    let mut area = 0;
    let mut coords = first
        .iter()
        .chain(rest)
        .chain(first.iter())
        .map(|&x| ParameterInteger(x).value());
    cursor[0] += coords.next().unwrap();
    cursor[1] += coords.next().unwrap();
    for i in 0..nb {
        let [x0, y0] = cursor;
        cursor[0] += coords.next().unwrap();
        cursor[1] += coords.next().unwrap();
        area += x0 * cursor[1] - cursor[0] * y0;
    }
    area > 0
}

#[cfg(test)]
#[cfg(feature = "with-geojson")]
mod test {
    use super::*;
    use crate::geojson::GeoJsonWriter;
    use crate::mvt::vector_tile::Tile;
    use crate::wkt::WktWriter;
    use crate::{ProcessToJson, ProcessToSvg, ToJson, ToWkt};
    use std::fs::File;

    #[test]
    fn layer() {
        // https://github.com/mapbox/vector-tile-spec/tree/master/2.1#45-example
        let mut mvt_layer = tile::Layer::default();
        mvt_layer.version = 2;
        mvt_layer.name = String::from("points");
        mvt_layer.extent = Some(4096);

        mvt_layer.keys.push(String::from("hello"));
        mvt_layer.keys.push(String::from("h"));
        mvt_layer.keys.push(String::from("count"));

        mvt_layer.values.push(tile::Value {
            string_value: Some(String::from("world")),
            ..Default::default()
        });
        mvt_layer.values.push(tile::Value {
            double_value: Some(1.23),
            ..Default::default()
        });
        mvt_layer.values.push(tile::Value {
            string_value: Some(String::from("again")),
            ..Default::default()
        });
        mvt_layer.values.push(tile::Value {
            int_value: Some(2),
            ..Default::default()
        });

        {
            let mut mvt_feature = tile::Feature::default();
            mvt_feature.id = Some(1);
            mvt_feature.set_type(GeomType::Point);
            mvt_feature.tags = [0, 0, 1, 0, 2, 1].to_vec();
            mvt_feature.geometry = [9, 2410, 3080].to_vec();
            mvt_layer.features.push(mvt_feature);
        }

        {
            let mut mvt_feature = tile::Feature::default();
            mvt_feature.id = Some(2);
            mvt_feature.set_type(GeomType::Point);
            mvt_feature.tags = [0, 2, 2, 3].to_vec();
            mvt_feature.geometry = [9, 2410, 3080].to_vec();
            mvt_layer.features.push(mvt_feature);
        }

        let geojson = mvt_layer.to_json().unwrap();

        assert_eq!(
            geojson.replace("\n", "").replace(" ", ""),
            r#"{
    "type": "FeatureCollection",
    "name": "points",
    "features": [
        {
            "type": "Feature",
            "properties": {
                "hello": "world",
                "h": "world",
                "count": 1.23
            },
            "geometry": {
                "type": "Point",
                "coordinates": [1205,1540]
            }
        },
        {
            "type": "Feature",
            "properties": {
                "hello": "again",
                "count": 2
            },
            "geometry": {
                "type": "Point",
                "coordinates": [1205,1540]
            }
        }
    ]
}"#
            .replace("\n", "")
            .replace(" ", "")
        );
    }

    #[test]
    fn point_geom() {
        let mut mvt_feature = tile::Feature::default();
        mvt_feature.set_type(GeomType::Point);
        mvt_feature.geometry = [9, 50, 34].to_vec();

        let geojson = mvt_feature.to_json().unwrap();

        assert_eq!(geojson, r#"{"type": "Point", "coordinates": [25,17]}"#);
    }

    #[test]
    fn multipoint_geom() {
        let mut mvt_feature = tile::Feature::default();
        mvt_feature.set_type(GeomType::Point);
        mvt_feature.geometry = [17, 10, 14, 3, 9].to_vec();

        let geojson = mvt_feature.to_json().unwrap();

        assert_eq!(
            geojson,
            r#"{"type": "MultiPoint", "coordinates": [[5,7],[3,2]]}"#
        );
    }

    #[test]
    fn line_geom() {
        let mut mvt_feature = tile::Feature::default();
        mvt_feature.set_type(GeomType::Linestring);
        mvt_feature.geometry = [9, 4, 4, 18, 0, 16, 16, 0].to_vec();

        let geojson = mvt_feature.to_json().unwrap();

        assert_eq!(
            geojson,
            r#"{"type": "LineString", "coordinates": [[2,2],[2,10],[10,10]]}"#
        );
    }

    #[test]
    fn multiline_geom() {
        let mut mvt_feature = tile::Feature::default();
        mvt_feature.set_type(GeomType::Linestring);
        mvt_feature.geometry = [9, 4, 4, 18, 0, 16, 16, 0, 9, 17, 17, 10, 4, 8].to_vec();

        let geojson = mvt_feature.to_json().unwrap();

        assert_eq!(
            geojson,
            r#"{"type": "MultiLineString", "coordinates": [[[2,2],[2,10],[10,10]],[[1,1],[3,5]]]}"#
        );
    }

    #[test]
    fn polygon_geom() {
        let mut mvt_feature = tile::Feature::default();
        mvt_feature.set_type(GeomType::Polygon);
        mvt_feature.geometry = [9, 6, 12, 18, 10, 12, 24, 44, 15].to_vec();

        let geojson = mvt_feature.to_json().unwrap();

        assert_eq!(
            geojson,
            r#"{"type": "Polygon", "coordinates": [[[3,6],[8,12],[20,34],[3,6]]]}"#
        );
    }

    #[test]
    fn multipolygon_geom() {
        let mut mvt_feature = tile::Feature::default();
        mvt_feature.set_type(GeomType::Polygon);
        mvt_feature.geometry = [
            9, 0, 0, 26, 20, 0, 0, 20, 19, 0, 15, 9, 22, 2, 26, 18, 0, 0, 18, 17, 0, 15, 9, 4, 13,
            26, 0, 8, 8, 0, 0, 7, 15,
        ]
        .to_vec();

        let geojson = mvt_feature.to_json().unwrap();

        assert_eq!(
            geojson,
            r#"{"type": "MultiPolygon", "coordinates": [[[[0,0],[10,0],[10,10],[0,10],[0,0]]],[[[11,11],[20,11],[20,20],[11,20],[11,11]],[[13,13],[13,17],[17,17],[17,13],[13,13]]]]}"#
        );
    }
}
