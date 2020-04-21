use geojson::{GeoJson, Geometry, Value};
use geozero::error::{GeozeroError, Result};
use geozero::GeomProcessor;
use std::io::Read;

pub fn read_geojson<R: Read, P: GeomProcessor>(mut reader: R, processor: &mut P) -> Result<()> {
    let mut geojson_str = String::new();
    reader.read_to_string(&mut geojson_str)?;
    let geojson = geojson_str
        .parse::<GeoJson>()
        .map_err(|_| GeozeroError::GeometryFormat)?;
    process_geojson(&geojson, processor)
}

/// Process top-level GeoJSON items
fn process_geojson<P: GeomProcessor>(gj: &GeoJson, processor: &mut P) -> Result<()> {
    match *gj {
        GeoJson::FeatureCollection(ref collection) => {
            for (idx, geometry) in collection
                .features
                .iter()
                // Only pass on non-empty geometries, doing so by reference
                .filter_map(|feature| feature.geometry.as_ref())
                .enumerate()
            {
                match_geometry(geometry, idx, processor)?;
            }
        }
        GeoJson::Feature(ref feature) => {
            if let Some(ref geometry) = feature.geometry {
                match_geometry(geometry, 0, processor)?;
            }
        }
        GeoJson::Geometry(ref geometry) => {
            match_geometry(geometry, 0, processor)?;
        }
    }
    Ok(())
}

/// Process GeoJSON geometries
fn match_geometry<P: GeomProcessor>(geom: &Geometry, idx: usize, processor: &mut P) -> Result<()> {
    match geom.value {
        Value::Point(ref geometry) => {
            process_point(geometry, idx, processor)?;
        }
        Value::MultiPoint(ref geometry) => {
            process_multi_point(geometry, idx, processor)?;
        }
        Value::LineString(ref geometry) => {
            process_linestring(geometry, true, idx, processor)?;
        }
        Value::MultiLineString(ref geometry) => {
            process_multilinestring(geometry, idx, processor)?;
        }
        Value::Polygon(ref geometry) => {
            process_polygon(geometry, true, idx, processor)?;
        }
        Value::MultiPolygon(ref geometry) => {
            process_multi_polygon(geometry, idx, processor)?;
        }
        Value::GeometryCollection(ref collection) => {
            // processor.geomcollection_begin(collection.len());
            for (idx, geometry) in collection.iter().enumerate() {
                match_geometry(geometry, idx, processor)?;
            }
        }
    }
    Ok(())
}

type Position = Vec<f64>;
type PointType = Position;
type LineStringType = Vec<Position>;
type PolygonType = Vec<Vec<Position>>;

fn process_point<P: GeomProcessor>(
    point_type: &PointType,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.point_begin(idx)?;
    processor.xy(point_type[0], point_type[1], 0)?;
    processor.point_end(idx)
}

fn process_multi_point<P: GeomProcessor>(
    multi_point_type: &[PointType],
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.multipoint_begin(multi_point_type.len(), idx)?;
    for (idxc, point_type) in multi_point_type.iter().enumerate() {
        process_point(&point_type, idxc, processor)?;
    }
    processor.multipoint_end(idx)
}

fn process_linestring<P: GeomProcessor>(
    linestring_type: &LineStringType,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.linestring_begin(tagged, linestring_type.len(), idx)?;
    for (idxc, point_type) in linestring_type.iter().enumerate() {
        processor.xy(point_type[0], point_type[1], idxc)?;
    }
    processor.linestring_end(tagged, idx)
}

fn process_multilinestring<P: GeomProcessor>(
    multilinestring_type: &[LineStringType],
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.multilinestring_begin(multilinestring_type.len(), idx)?;
    for (idxc, linestring_type) in multilinestring_type.iter().enumerate() {
        process_linestring(&linestring_type, false, idxc, processor)?
    }
    processor.multilinestring_end(idx)
}

fn process_polygon<P: GeomProcessor>(
    polygon_type: &PolygonType,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.polygon_begin(tagged, polygon_type.len(), idx)?;
    for (idxl, linestring_type) in polygon_type.iter().enumerate() {
        process_linestring(linestring_type, false, idxl, processor)?
    }
    processor.polygon_end(tagged, idx)
}

fn process_multi_polygon<P: GeomProcessor>(
    multi_polygon_type: &[PolygonType],
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.multipolygon_begin(multi_polygon_type.len(), idx)?;
    for (idxp, polygon_type) in multi_polygon_type.iter().enumerate() {
        process_polygon(&polygon_type, false, idxp, processor)?;
    }
    processor.multipolygon_end(idx)
}

#[test]
#[ignore]
fn from_file() -> Result<()> {
    use crate::wkt_writer::WktWriter;
    use std::fs::File;

    let f = File::open("tests/data/canada.json")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    read_geojson(f, &mut WktWriter::new(&mut wkt_data))?;
    let wkt = std::str::from_utf8(&wkt_data).unwrap();
    assert_eq!(
        &wkt[0..100],
        "POLYGON ((-65.61361699999998 43.42027300000001, -65.61972000000003 43.418052999999986, -65.625 43.42"
    );
    assert_eq!(
        &wkt[wkt.len()-100..],
        "9997 83.11387600000012, -70.16000399999996 83.11137400000001, -70.11193799999995 83.10942100000011))"
    );
    Ok(())
}
