mod driver;

use crate::driver::{Extent, HttpReader, Reader, SelectOpts};
use flatgeobuf::*;
use geozero::error::Result;
use geozero::geojson::{GeoJson, GeoJsonReader, GeoJsonWriter};
use geozero::svg::SvgWriter;
use geozero::{FeatureProcessor, GeozeroDatasource};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::num::ParseFloatError;
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    /// The path or URL to the FlatGeobuf file to read
    input: String,
    /// Geometries within extent
    #[structopt(short, long, parse(try_from_str = parse_extent))]
    extent: Option<Extent>,
    /// The path to the file to write
    #[structopt(parse(from_os_str))]
    dest: std::path::PathBuf,
}

fn parse_extent(src: &str) -> std::result::Result<Extent, ParseFloatError> {
    let arr: Vec<f64> = src
        .split(",")
        .map(|v| {
            v.parse()
                .expect("Error parsing 'extent' as list of float values")
        })
        .collect();
    Ok(Extent {
        minx: arr[0],
        miny: arr[1],
        maxx: arr[2],
        maxy: arr[3],
    })
}

fn transform<P: FeatureProcessor>(args: Cli, processor: &mut P) -> Result<()> {
    let pathin = Path::new(&args.input);
    let select_opts = SelectOpts {
        extent: args.extent,
    };
    let mut filein = BufReader::new(File::open(pathin)?);
    match pathin.extension().and_then(OsStr::to_str) {
        Some("fgb") => {
            let mut ds = FgbReader::open(&mut filein)?;
            ds.select(&select_opts)?;
            GeozeroDatasource::process(&mut ds, processor)?;
        }
        Some("json") | Some("geojson") => {
            let mut ds = GeoJsonReader(&mut filein);
            GeozeroDatasource::process(&mut ds, processor)?;
        }
        _ => panic!("Unkown input file extension"),
    };
    Ok(())
}

fn process(args: Cli) -> Result<()> {
    let mut fout = BufWriter::new(File::create(&args.dest)?);
    match args.dest.extension().and_then(OsStr::to_str) {
        Some("json") | Some("geojson") => {
            let mut processor = GeoJsonWriter::new(&mut fout);
            transform(args, &mut processor)?;
        }
        Some("svg") => {
            let mut processor = SvgWriter::new(&mut fout, true);
            if let Some(extent) = args.extent {
                processor.set_dimensions(
                    extent.minx,
                    extent.miny,
                    extent.maxx,
                    extent.maxy,
                    800,
                    600,
                );
            } else {
                // TODO: get image size as opts and full extent from data
                processor.set_dimensions(-180.0, -90.0, 180.0, 90.0, 800, 600);
            }
            transform(args, &mut processor)?;
        }
        Some("fgb") => {
            let mut fgb = FgbWriter::create("countries", GeometryType::MultiPolygon, None, |_| {})?;
            // transform(args, &mut fgb)?;
            let geojson = GeoJson(
                r#"{"type": "Feature", "properties": {"fid": 42, "name": "New Zealand"}, "geometry": {"type": "MultiPolygon", "coordinates": [[[[173.020375,-40.919052],[173.247234,-41.331999],[173.958405,-40.926701],[174.247587,-41.349155],[174.248517,-41.770008],[173.876447,-42.233184],[173.22274,-42.970038],[172.711246,-43.372288],[173.080113,-43.853344],[172.308584,-43.865694],[171.452925,-44.242519],[171.185138,-44.897104],[170.616697,-45.908929],[169.831422,-46.355775],[169.332331,-46.641235],[168.411354,-46.619945],[167.763745,-46.290197],[166.676886,-46.219917],[166.509144,-45.852705],[167.046424,-45.110941],[168.303763,-44.123973],[168.949409,-43.935819],[169.667815,-43.555326],[170.52492,-43.031688],[171.12509,-42.512754],[171.569714,-41.767424],[171.948709,-41.514417],[172.097227,-40.956104],[172.79858,-40.493962],[173.020375,-40.919052]]],[[[174.612009,-36.156397],[175.336616,-37.209098],[175.357596,-36.526194],[175.808887,-36.798942],[175.95849,-37.555382],[176.763195,-37.881253],[177.438813,-37.961248],[178.010354,-37.579825],[178.517094,-37.695373],[178.274731,-38.582813],[177.97046,-39.166343],[177.206993,-39.145776],[176.939981,-39.449736],[177.032946,-39.879943],[176.885824,-40.065978],[176.508017,-40.604808],[176.01244,-41.289624],[175.239567,-41.688308],[175.067898,-41.425895],[174.650973,-41.281821],[175.22763,-40.459236],[174.900157,-39.908933],[173.824047,-39.508854],[173.852262,-39.146602],[174.574802,-38.797683],[174.743474,-38.027808],[174.697017,-37.381129],[174.292028,-36.711092],[174.319004,-36.534824],[173.840997,-36.121981],[173.054171,-35.237125],[172.636005,-34.529107],[173.007042,-34.450662],[173.551298,-35.006183],[174.32939,-35.265496],[174.612009,-36.156397]]]]}}"#,
            );
            fgb.add_feature(geojson).ok();
            fgb.write(&mut fout)?;
        }
        _ => panic!("Unkown output file extension"),
    }
    Ok(())
}

#[tokio::main]
async fn process_url(args: Cli) -> Result<()> {
    let mut ds = HttpFgbReader::open(&args.input).await?;

    let select_opts = SelectOpts {
        extent: args.extent,
    };
    ds.select(&select_opts).await?;

    let mut fout = BufWriter::new(File::create(&args.dest)?);
    match args.dest.extension().and_then(OsStr::to_str) {
        Some("json") | Some("geojson") => {
            let mut processor = GeoJsonWriter::new(&mut fout);
            ds.process(&mut processor).await?;
        }
        Some("svg") => {
            let mut processor = SvgWriter::new(&mut fout, true);
            if let Some(extent) = args.extent {
                processor.set_dimensions(
                    extent.minx,
                    extent.miny,
                    extent.maxx,
                    extent.maxy,
                    800,
                    600,
                );
            } else {
                processor.set_dimensions(-180.0, -90.0, 180.0, 90.0, 800, 600);
            }
            ds.process(&mut processor).await?;
        }
        _ => panic!("Unkown output format"),
    }
    Ok(())
}

fn main() {
    let args = Cli::from_args();

    let result = if args.input.starts_with("http") {
        process_url(args).map_err(|e| e.to_string())
    } else {
        process(args).map_err(|e| e.to_string())
    };
    if let Err(msg) = result {
        println!("Processing failed: {}", msg);
    }
}
