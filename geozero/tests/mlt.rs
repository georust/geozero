use geozero::geojson::GeoJson;
use geozero::mlt::{MltReader, MltWriter};
use geozero::mvt::{Message, Tile};
use geozero::{GeozeroDatasource, ProcessToJson};

#[test]
fn mlt_fixture_matches_mvt() {
    let maplibre_tile = &include_bytes!("data/tile.mlt")[..];
    let reader = MltReader::new(maplibre_tile).unwrap();
    let decoded: Vec<String> = reader
        .layers()
        .map(|mut layer| layer.to_json().unwrap())
        .collect();

    let vector_tile = &include_bytes!("data/tile.mvt")[..];
    let tile = Tile::decode(vector_tile).unwrap();
    let reference: Vec<String> = tile
        .layers
        .into_iter()
        .map(|mut layer| layer.to_json().unwrap())
        .collect();

    assert_eq!(reader.layer_names().collect::<Vec<_>>(), vec!["cities"]);
    assert_eq!(decoded, reference);
}

#[test]
fn mlt_write_read_roundtrip() {
    let input = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": { "name": "point" },
                "geometry": { "type": "Point", "coordinates": [10, 20] }
            },
            {
                "type": "Feature",
                "properties": { "name": "line" },
                "geometry": { "type": "LineString", "coordinates": [[1, 2], [3, 4], [5, 6]] }
            },
            {
                "type": "Feature",
                "properties": { "name": "multipoint" },
                "geometry": { "type": "MultiPoint", "coordinates": [[7, 8], [9, 10]] }
            }
        ]
    }"#;

    let mut writer = MltWriter::new("roundtrip", 4096).unwrap();
    GeoJson(input).process(&mut writer).unwrap();
    let mlt_bytes = writer.finish().unwrap();

    let reader = MltReader::new(&mlt_bytes).unwrap();
    let mut layers = reader.layers();
    let mut layer = layers.next().expect("one layer");
    let output = layer.to_json().unwrap();
    assert!(layers.next().is_none());

    let input_json: serde_json::Value = serde_json::from_str(input).unwrap();
    let output_json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let in_features = input_json["features"].as_array().unwrap();
    let out_features = output_json["features"].as_array().unwrap();
    assert_eq!(in_features.len(), out_features.len());

    for (expected, actual) in in_features.iter().zip(out_features) {
        assert_eq!(expected["geometry"], actual["geometry"]);
        assert_eq!(expected["properties"]["name"], actual["properties"]["name"]);
    }
}
