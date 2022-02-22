use crate::error::Result;
use crate::mvt::vector_tile::{tile, tile::GeomType};
use crate::{GeomProcessor, GeozeroGeometry};

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
    match geom.r#type {
        // Some(GeomType::Point) => {
        //     processor.point_begin(idx)?;
        //     process_coord(geom, 0, processor)?;
        //     processor.point_end(idx)?;
        // }
        // Some(GeomType::Linestring) => {
        //     process_linestring(geom, true, idx, processor)?;
        // }
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
    coord: &tile::Feature,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    // if processor.multi_dim() {
    //     processor.coordinate(coord.x, coord.y, None, None, None, None, idx)?;
    // } else {
    //     processor.xy(coord.x, coord.y, idx)?;
    // }
    Ok(())
}

fn process_linestring<P: GeomProcessor>(
    geom: &tile::Feature,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let multi = processor.multi_dim();
    // processor.linestring_begin(tagged, geom.0.len(), idx)?;
    // for (i, coord) in geom.0.iter().enumerate() {
    //     if multi {
    //         processor.coordinate(coord.x, coord.y, None, None, None, None, i)?;
    //     } else {
    //         processor.xy(coord.x, coord.y, i)?;
    //     }
    // }
    processor.linestring_end(tagged, idx)
}

fn process_polygon<P: GeomProcessor>(
    geom: &tile::Feature,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    // let interiors = geom.interiors();
    // processor.polygon_begin(tagged, interiors.len() + 1, idx)?;
    // // Exterior ring
    // process_linestring(&geom.exterior(), false, 0, processor)?;
    // // Interior rings
    // for (i, ring) in interiors.iter().enumerate() {
    //     process_linestring(&ring, false, i + 1, processor)?;
    // }
    processor.polygon_end(tagged, idx)
}

#[cfg(test)]
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
        mvt_feature.id = Some(1);
        mvt_feature.set_type(GeomType::Point);
        mvt_feature.geometry = [9, 50, 34].to_vec();

        let geojson = mvt_feature.to_json().unwrap();

        assert_eq!(geojson, r#"{"type": "Point", "coordinates": [25, 17]}"#);
    }
}
