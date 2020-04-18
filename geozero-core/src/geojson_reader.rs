use geojson::{GeoJson, Geometry, Value};

/// Process GeoJSON geometries
fn match_geometry(geom: &Geometry) {
    match geom.value {
        Value::Polygon(_) => println!("Matched a Polygon"),
        Value::MultiPolygon(_) => println!("Matched a MultiPolygon"),
        Value::GeometryCollection(ref collection) => {
            println!("Matched a GeometryCollection");
            // GeometryCollections contain other Geometry types, and can nest
            // we deal with this by recursively processing each geometry
            collection
                .iter()
                .for_each(|geometry| match_geometry(geometry))
        }
        // Point, LineString, and their Multi– counterparts
        _ => println!("Matched some other geometry"),
    }
}

/// Process top-level GeoJSON items
fn process_geojson(gj: &GeoJson) {
    match *gj {
        GeoJson::FeatureCollection(ref collection) => collection
            .features
            .iter()
            // Only pass on non-empty geometries, doing so by reference
            .filter_map(|feature| feature.geometry.as_ref())
            .for_each(|geometry| match_geometry(geometry)),
        GeoJson::Feature(ref feature) => {
            if let Some(ref geometry) = feature.geometry {
                match_geometry(geometry)
            }
        }
        GeoJson::Geometry(ref geometry) => match_geometry(geometry),
    }
}

#[test]
fn main() {
    let geojson_str = include!("test.geojson");
    let geojson = geojson_str.parse::<GeoJson>().unwrap();
    process_geojson(&geojson);
}
