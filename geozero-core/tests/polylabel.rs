use flatgeobuf::*;
use geo::contains::Contains;
use geo::Geometry;
use geozero::error::Result;
use geozero_core::geo_types::Geo;
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
        let geometry = feature.geometry().unwrap();
        let mut geo = Geo::new();
        geometry.process(&mut geo, GeometryType::MultiPolygon)?;
        if let Geometry::MultiPolygon(mpoly) = geo.geometry() {
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
