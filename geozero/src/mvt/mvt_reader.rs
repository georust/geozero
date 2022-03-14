use crate::error::{GeozeroError, Result};
use crate::mvt::vector_tile::{tile, tile::GeomType};
use crate::{GeomProcessor, GeozeroGeometry};

use super::mvt_commands::{Command, CommandInteger, ParameterInteger};

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
        // Some(GeomType::Polygon) => {
        //     process_polygon(geom, true, idx, processor)?;
        // }
        // Some(GeomType::Unknown) => {
        //     todo!()
        // }
        // None => {
        //     todo!()
        // }
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

    let multi_linestring = line_string_slices.len() > 1;

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

#[cfg(test)]
#[cfg(feature = "with-geojson")]
mod test {
    use super::*;
    use crate::geojson::GeoJsonWriter;
    use crate::mvt::vector_tile::Tile;
    use crate::wkt::WktWriter;
    use crate::{ProcessToSvg, ToJson, ToWkt};
    use std::fs::File;

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
}
