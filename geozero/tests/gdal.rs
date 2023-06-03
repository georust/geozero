use gdal::vector::LayerAccess;
use gdal::Dataset;
use geozero::gdal::process_geom;
use geozero::svg::SvgWriter;
use geozero::ToSvg;
use std::path::Path;

#[test]
fn ogr_to_svg() -> Result<(), gdal::errors::GdalError> {
    let dataset = Dataset::open(Path::new("tests/data/places.json"))?;
    let mut layer = dataset.layer(0)?;
    let mut out: Vec<u8> = Vec::new();
    for feature in layer.features() {
        let geom = feature.geometry().unwrap();
        let svg = geom.to_svg();
        assert!(svg.is_ok());
        // concatenate SVG geometries
        assert!(process_geom(geom, &mut SvgWriter::new(&mut out, true)).is_ok());
    }
    assert_eq!(
        &std::str::from_utf8(&out).unwrap()[..53],
        r#"<path d="M 32.533299524864844 -0.583299105614628 Z"/>"#
    );
    Ok(())
}
