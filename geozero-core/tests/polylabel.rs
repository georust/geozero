use flatgeobuf::*;
use geo::contains::Contains;
use geo::Geometry;
use geozero::error::Result;
use geozero_core::geo::RustGeo;
use polylabel::polylabel;
use std::fs::File;
use std::io::BufReader;

#[test]
fn country_labels() -> Result<()> {
    let mut file = BufReader::new(File::open("tests/data/countries.fgb")?);
    let hreader = HeaderReader::read(&mut file)?;
    let header = hreader.header();

    let mut freader = FeatureReader::select_all(&mut file, &header)?;
    while let Some(feature) = freader.next(&mut file)? {
        let props = feature.properties_map(&header)?;
        let geometry = feature.geometry().unwrap();
        let mut geo = RustGeo::new();
        geometry.process(&mut geo, header.geometry_type())?;
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
