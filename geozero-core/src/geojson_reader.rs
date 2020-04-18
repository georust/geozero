use geojson::{GeoJson, Geometry, Value};
use geozero::GeomProcessor;
use std::io::Read;

pub fn read_geojson<R: Read, P: GeomProcessor>(
    mut reader: R,
    processor: &mut P,
) -> std::result::Result<(), std::io::Error> {
    let mut geojson_str = String::new();
    reader.read_to_string(&mut geojson_str)?;
    let geojson = geojson_str.parse::<GeoJson>().unwrap();
    process_geojson(&geojson, processor);
    Ok(())
}

/// Process top-level GeoJSON items
fn process_geojson<P: GeomProcessor>(gj: &GeoJson, processor: &mut P) {
    match *gj {
        GeoJson::FeatureCollection(ref collection) => collection
            .features
            .iter()
            // Only pass on non-empty geometries, doing so by reference
            .filter_map(|feature| feature.geometry.as_ref())
            .for_each(|geometry| match_geometry(geometry, processor)),
        GeoJson::Feature(ref feature) => {
            if let Some(ref geometry) = feature.geometry {
                match_geometry(geometry, processor)
            }
        }
        GeoJson::Geometry(ref geometry) => match_geometry(geometry, processor),
    }
}

/// Process GeoJSON geometries
fn match_geometry<P: GeomProcessor>(geom: &Geometry, processor: &mut P) {
    match geom.value {
        Value::Point(ref geometry) => process_point(geometry, processor),
        Value::MultiPoint(ref geometry) => process_multi_point(geometry, processor),
        Value::LineString(ref geometry) => process_line_string(geometry, processor),
        Value::MultiLineString(ref geometry) => process_multi_line_string(geometry, processor),
        Value::Polygon(ref geometry) => {
            process_polygon(geometry, processor);
        }
        Value::MultiPolygon(ref geometry) => {
            process_multi_polygon(geometry, processor);
        }
        Value::GeometryCollection(ref collection) => {
            println!("Matched a GeometryCollection");
            collection
                .iter()
                .for_each(|geometry| match_geometry(geometry, processor))
        }
    }
}

type Position = Vec<f64>;
type PointType = Position;
type LineStringType = Vec<Position>;
type PolygonType = Vec<Vec<Position>>;

fn process_coordinate<P: GeomProcessor>(point_type: &PointType, processor: &mut P) {
    processor.pointxy(point_type[0], point_type[1], 0);
}

fn process_point<P: GeomProcessor>(point_type: &PointType, processor: &mut P) {
    processor.pointxy(point_type[0], point_type[1], 0);
}

fn process_multi_point<P: GeomProcessor>(multi_point_type: &[PointType], processor: &mut P) {
    multi_point_type
        .iter()
        .for_each(|point_type| process_point(&point_type, processor));
}

fn process_line_string<P: GeomProcessor>(line_type: &LineStringType, processor: &mut P) {
    line_type
        .iter()
        .for_each(|point_type| process_coordinate(point_type, processor));
}

fn process_multi_line_string<P: GeomProcessor>(
    multi_line_type: &[LineStringType],
    processor: &mut P,
) {
    multi_line_type
        .iter()
        .for_each(|point_type| process_line_string(&point_type, processor));
}

fn process_polygon<P: GeomProcessor>(polygon_type: &PolygonType, processor: &mut P) {
    polygon_type
        .iter()
        .for_each(|line_string_type| process_line_string(line_string_type, processor));
}

fn process_multi_polygon<P: GeomProcessor>(multi_polygon_type: &[PolygonType], processor: &mut P) {
    multi_polygon_type
        .iter()
        .for_each(|polygon_type| process_polygon(&polygon_type, processor));
}

#[test]
fn from_file() -> std::result::Result<(), std::io::Error> {
    use std::fs::File;

    let f = File::open("canada.json")?;
    read_geojson(f, &mut geozero::DebugReader {})?;
    assert!(false);
    Ok(())
}
