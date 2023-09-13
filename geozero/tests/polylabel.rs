use flatgeobuf::{FallibleStreamingIterator, FeatureProperties, FgbReader};
use geo::contains::Contains;
use geo::Geometry;
use geozero::error::Result;
use geozero::ToGeo;
use polylabel::polylabel;
use seek_bufread::BufReader;
use std::fs::File;

#[test]
fn country_labels() -> Result<()> {
    let mut file = BufReader::new(File::open("tests/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut file)?.select_all()?;
    while let Some(feature) = fgb.next()? {
        let name: String = feature.property("name").unwrap();
        if let Ok(Geometry::MultiPolygon(mpoly)) = feature.to_geo() {
            if let Some(poly) = &mpoly.0.first() {
                let label_pos = polylabel(poly, &0.10).unwrap();
                println!("{name}: {label_pos:?}");
                if !["Bermuda", "Falkland Islands"].contains(&name.as_str()) {
                    assert!(mpoly.contains(&label_pos));
                }
            }
        }
    }
    Ok(())
}
