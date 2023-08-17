use clap::Parser;
use flatgeobuf::{FgbReader, FgbWriter, GeometryType, HttpFgbReader};
use geozero::csv::{CsvReader, CsvWriter};
use geozero::error::Result;
use geozero::geojson::{GeoJsonReader, GeoJsonWriter};
use geozero::svg::SvgWriter;
use geozero::wkt::{WktReader, WktWriter};
use geozero::{FeatureProcessor, GeozeroDatasource};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::num::ParseFloatError;
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(Parser)]
#[command(about, version)]
struct Cli {
    /// When processing CSV, the name of the column holding a WKT geometry.
    #[arg(long)]
    csv_geometry_column: Option<String>,

    /// Geometries within extent
    #[arg(short, long, value_parser = parse_extent)]
    extent: Option<Extent>,

    /// The path or URL to the FlatGeobuf file to read
    input: String,

    /// The path to the file to write
    dest: PathBuf,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Extent {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

fn parse_extent(src: &str) -> std::result::Result<Extent, ParseFloatError> {
    let arr: Vec<f64> = src
        .split(',')
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
    let path_in = Path::new(&args.input);
    let mut filein = BufReader::new(File::open(path_in)?);
    match path_in.extension().and_then(OsStr::to_str) {
        Some("csv") => {
            let geometry_column_name = args
                .csv_geometry_column
                .expect("must specify --csv-geometry-column=<column name> when parsing CSV");
            let mut ds = CsvReader::new(&geometry_column_name, &mut filein);
            GeozeroDatasource::process(&mut ds, processor)
        }
        Some("json") | Some("geojson") => {
            GeozeroDatasource::process(&mut GeoJsonReader(filein), processor)
        }
        Some("fgb") => {
            let ds = FgbReader::open(&mut filein)?;
            let mut ds = if let Some(bbox) = &args.extent {
                ds.select_bbox(bbox.minx, bbox.miny, bbox.maxx, bbox.maxy)?
            } else {
                ds.select_all()?
            };
            ds.process_features(processor)
        }
        Some("wkt") => GeozeroDatasource::process(&mut WktReader(&mut filein), processor),
        _ => panic!("Unknown input file extension"),
    }
}

fn process(args: Cli) -> Result<()> {
    let mut fout = BufWriter::new(File::create(&args.dest)?);
    match args.dest.extension().and_then(OsStr::to_str) {
        Some("csv") => transform(args, &mut CsvWriter::new(&mut fout)),
        Some("wkt") => transform(args, &mut WktWriter::new(&mut fout)),
        Some("json") | Some("geojson") => transform(args, &mut GeoJsonWriter::new(&mut fout)),
        Some("fgb") => {
            let mut fgb = FgbWriter::create("fgb", GeometryType::Unknown)?;
            transform(args, &mut fgb)?;
            fgb.write(&mut fout)
        }
        Some("svg") => {
            let mut processor = SvgWriter::new(&mut fout, true);
            set_dimensions(&mut processor, args.extent);
            transform(args, &mut processor)
        }
        _ => panic!("Unknown output file extension"),
    }
}

#[tokio::main]
async fn process_url(args: Cli) -> Result<()> {
    let ds = HttpFgbReader::open(&args.input).await?;
    let mut ds = if let Some(bbox) = &args.extent {
        ds.select_bbox(bbox.minx, bbox.miny, bbox.maxx, bbox.maxy)
            .await?
    } else {
        ds.select_all().await?
    };

    let mut fout = BufWriter::new(File::create(&args.dest)?);
    match args.dest.extension().and_then(OsStr::to_str) {
        Some("json") | Some("geojson") => {
            let mut processor = GeoJsonWriter::new(&mut fout);
            ds.process_features(&mut processor).await
        }
        Some("svg") => {
            let mut processor = SvgWriter::new(&mut fout, true);
            set_dimensions(&mut processor, args.extent);
            ds.process_features(&mut processor).await
        }
        _ => panic!("Unknown output format"),
    }
}

fn set_dimensions(processor: &mut SvgWriter<&mut BufWriter<File>>, extent: Option<Extent>) {
    if let Some(extent) = extent {
        processor.set_dimensions(extent.minx, extent.miny, extent.maxx, extent.maxy, 800, 600);
    } else {
        // TODO: get image size as opts and full extent from data
        processor.set_dimensions(-180.0, -90.0, 180.0, 90.0, 800, 600);
    }
}

fn main() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env).init();

    let args = Cli::parse();

    let result = if args.input.starts_with("http") {
        process_url(args)
    } else {
        process(args)
    };
    if let Err(msg) = result {
        println!("Processing failed: {msg}");
        exit(1)
    }
}
