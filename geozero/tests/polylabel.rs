use flatgeobuf::{FallibleStreamingIterator, FeatureProperties, FgbReader};
use geo::contains::Contains;
use geo::Geometry;
use geozero::ToGeo;
use geozero::error::Result;
use polylabel::polylabel;
use std::fs::File;
use std::io::BufReader;

#[test]
fn country_labels() -> Result<()> {
    let mut file = BufReader::new(File::open("tests/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut file)?;
    fgb.select_all()?;
    while let Some(feature) = fgb.next()? {
        let props = feature.properties()?;
        if let Ok(Geometry::MultiPolygon(mpoly)) = feature.to_geo() {
            if let Some(poly) = &mpoly.0.iter().next() {
                let label_pos = polylabel(&poly, &0.10).unwrap();
                println!("{}: {:?}", props["name"], label_pos);
                if !vec!["Bermuda", "Falkland Islands"].contains(&props["name"].as_str()) {
                    assert!(mpoly.contains(&label_pos));
                }
            }
        }
    }
    Ok(())
}
