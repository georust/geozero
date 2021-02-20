use flatgeobuf::*;
use geozero::error::Result;
use geozero::geojson::GeoJsonWriter;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[test]
fn fgb_to_geojson() -> Result<()> {
    let mut filein = BufReader::new(File::open("tests/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?;
    fgb.select_all()?;
    let mut json_data: Vec<u8> = Vec::new();
    let mut json = GeoJsonWriter::new(&mut json_data);
    fgb.process_features(&mut json)?;
    assert_eq!(
        &std::str::from_utf8(&json_data).unwrap()[0..215],
        r#"{
"type": "FeatureCollection",
"name": "countries",
"features": [{"type": "Feature", "properties": {"id": "ATA", "name": "Antarctica"}, "geometry": {"type": "MultiPolygon", "coordinates": [[[[-59.572095,-80.040179],"#
    );
    Ok(())
}

#[allow(dead_code)]
async fn http_fbg_to_json() -> Result<()> {
    let url = "https://github.com/georust/geozero/raw/master/geozero-core/tests/data/countries.fgb";
    let mut fgb = HttpFgbReader::open(url).await?;
    fgb.select_bbox(8.8, 47.2, 9.5, 55.3).await?;

    let mut fout = BufWriter::new(File::create("countries.json")?);
    let mut json = GeoJsonWriter::new(&mut fout);
    fgb.process_features(&mut json).await?;

    Ok(())
}

// #[test]
// fn http_json() {
//     assert!(tokio::runtime::Runtime::new()
//         .unwrap()
//         .block_on(http_fbg_to_json())
//         .is_ok());
// }
