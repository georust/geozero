mod driver;

use crate::driver::{Extent, HttpReader, Reader, SelectOpts};
use flatgeobuf::*;
use geozero::error::Result;
use geozero::geojson::{GeoJsonReader, GeoJsonWriter};
use geozero::svg::SvgWriter;
use geozero::{FeatureProcessor, GeozeroDatasource};
use std::env;
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
            let mut fgb = FgbWriter::create("fgb", GeometryType::Unknown, |_, _| {})?;
            transform(args, &mut fgb)?;
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

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let result = if args.input.starts_with("http") {
        process_url(args).map_err(|e| e.to_string())
    } else {
        process(args).map_err(|e| e.to_string())
    };
    if let Err(msg) = result {
        println!("Processing failed: {}", msg);
    }
}
