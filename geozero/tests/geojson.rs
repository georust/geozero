use flatgeobuf::{FgbReader, HttpFgbReader};
use geozero::error::Result;
use geozero::geojson::GeoJsonWriter;
use geozero::ProcessToJson;
use seek_bufread::BufReader;
use std::fs::File;
use std::io::BufWriter;

#[test]
fn fgb_to_geojson() -> Result<()> {
    let mut filein = BufReader::new(File::open("tests/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?.select_bbox(8.8, 47.2, 9.5, 55.3)?;
    let json = fgb.to_json()?;
    assert_eq!(
        &json[0..215],
        r#"{
"type": "FeatureCollection",
"name": "countries",
"features": [{"type": "Feature", "properties": {"id": "DNK", "name": "Denmark"}, "geometry": {"type": "MultiPolygon", "coordinates": [[[[12.690006,55.609991],[12.0"#
    );
    Ok(())
}

#[allow(dead_code)]
// #[tokio::test]
async fn http_fbg_to_json() -> Result<()> {
    let url = "https://flatgeobuf.org/test/data/countries.fgb";
    let mut fgb = HttpFgbReader::open(url)
        .await?
        .select_bbox(8.8, 47.2, 9.5, 55.3)
        .await?;

    let mut fout = BufWriter::new(File::create("countries.json")?);
    let mut json = GeoJsonWriter::new(&mut fout);
    fgb.process_features(&mut json).await?;

    Ok(())
}
