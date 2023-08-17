use flatgeobuf::{FgbReader, Header};
use geozero::error::Result;
use geozero::geojson::GeoJsonReader;
use geozero::svg::SvgWriter;
use geozero::ProcessToSvg;
use seek_bufread::BufReader;
use std::fs::File;
use std::io::Write;

#[test]
fn json_to_svg() -> Result<()> {
    let f = File::open("tests/data/places.json")?;
    let svg = GeoJsonReader(f).to_svg().unwrap();
    println!("{svg}");
    assert_eq!(
        &svg[svg.len() - 100..],
        r#"387481909902 1.294979325105942 Z"/>
<path d="M 114.18306345846304 22.30692675357551 Z"/>
</g>
</svg>"#
    );

    Ok(())
}

fn invert_y(header: &Header) -> bool {
    if let Some(crs) = header.crs() {
        if crs.code() == 4326 {
            return true;
        }
    }
    false
}

fn svg_writer<W: Write>(header: &Header, width: u32, height: u32, out: W) -> SvgWriter<W> {
    let mut svg = SvgWriter::new(out, invert_y(header));
    if let Some(envelope) = header.envelope() {
        svg.set_dimensions(
            envelope.get(0),
            envelope.get(1),
            envelope.get(2),
            envelope.get(3),
            width,
            height,
        );
    }
    svg
}

#[test]
fn fgb_to_svg() -> Result<()> {
    let mut filein = BufReader::new(File::open("tests/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?.select_bbox(8.8, 47.2, 9.5, 55.3)?;
    let mut svg_data: Vec<u8> = Vec::new();
    let mut svg = svg_writer(&fgb.header(), 800, 400, &mut svg_data);
    fgb.process_features(&mut svg)?;
    let out = std::str::from_utf8(&svg_data).unwrap();
    let expected = r#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" version="1.2" baseProfile="tiny" width="800" height="400" viewBox="-180 -83.64513 360 169.254168" stroke-linecap="round" stroke-linejoin="round">
<g id="countries">
<path d="M 12.690006 -55.609991 12.089991 -54.800015 11.043543 -55.364864 10.903914 -55.779955 12.370904 -56.111407 12.690006 -55.609991 Z "/><path d="M 10.912182 -56.458621 10.667804 -56.081383 10.369993 -56.190007 9.649985 -55.469999 9.921906 -54.983104 9.282049 -54.830865 8.526229 -54.962744 8.120311 -55.517723 8.089977 -56.540012 8.256582 -56.809969 8.543438 -57.110003 9.424469 -57.172066 9.775559 -57.447941 10.580006 -57.730017 10.546106 -57.215733 10.25 -56.890016 10.369993 -56.609982 10.912182 -56.458621 Z "/>
<path d="M 16.979667 -48.123497 16.903754 -47.714866 16.340584 -47.712902 16.534268 -47.496171 16.202298 -46.852386 16.011664 -46.683611 15.137092 -46.658703 14.632472 -46.431817 13.806475 -46.509306 12.376485 -46.767559 12.153088 -47.115393 11.164828 -46.941579 11.048556 -46.751359 10.442701 -46.893546 9.932448 -46.920728 9.47997 -47.10281 9.632932 -47.347601 9.594226 -47.525058 9.896068 -47.580197 10.402084 -47.302488 10.544504 -47.566399 11.426414 -47.523766 12.141357 -47.703083 12.62076 -47.672388 12.932627 -47.467646 13.025851 -47.637584 12.884103 -48.289146 13.243357 -48.416115 13.595946 -48.877172 14.338898 -48.555305 14.901447 -48.964402 15.253416 -49.039074 16.029647 -48.733899 16.499283 -48.785808 16.960288 -48.596982 16.879983 -48.470013 16.979667 -48.123497 Z "/>
"#;
    assert_eq!(&out[..expected.len()], expected);
    let expected = r#"99.93976 -78.88094 Z "/>
</g>
</svg>"#;
    assert_eq!(&out[svg_data.len() - expected.len()..], expected);

    Ok(())
}
