use clap::Parser;
use flatgeobuf::{FgbReader, FgbWriter, GeometryType, HttpFgbReader};
use geozero::csv::{CsvReader, CsvWriter};
use geozero::error::{GeozeroError, Result};
use geozero::geojson::{GeoJsonLineReader, GeoJsonReader, GeoJsonWriter};
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

    /// Will be placed within <style>...</style> tags of the top level svg element.
    #[arg(long)]
    svg_style: Option<String>,
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

async fn transform<P: FeatureProcessor>(args: &Cli, processor: &mut P) -> Result<()> {
    let path_in = Path::new(&args.input);
    if path_in.starts_with("http:") || path_in.starts_with("https:") {
        if path_in.extension().and_then(OsStr::to_str) != Some("fgb") {
            panic!("Remote access is only supported for .fgb input")
        }
        let ds = HttpFgbReader::open(&args.input)
            .await
            .map_err(fgb_to_geozero_err)?;
        let mut ds = if let Some(bbox) = &args.extent {
            ds.select_bbox(bbox.minx, bbox.miny, bbox.maxx, bbox.maxy)
                .await
                .map_err(fgb_to_geozero_err)?
        } else {
            ds.select_all().await.map_err(fgb_to_geozero_err)?
        };
        ds.process_features(processor).await
    } else {
        let mut filein = BufReader::new(File::open(path_in)?);
        match path_in.extension().and_then(OsStr::to_str) {
            Some("csv") => {
                let geometry_column_name = args
                    .csv_geometry_column
                    .as_ref()
                    .expect("must specify --csv-geometry-column=<column name> when parsing CSV");
                let mut ds = CsvReader::new(geometry_column_name, &mut filein);
                GeozeroDatasource::process(&mut ds, processor)
            }
            Some("json") | Some("geojson") => {
                GeozeroDatasource::process(&mut GeoJsonReader(filein), processor)
            }
            Some("jsonl") | Some("geojsonl") => {
                GeozeroDatasource::process(&mut GeoJsonLineReader::new(filein), processor)
            }
            Some("fgb") => {
                let ds = FgbReader::open(&mut filein).map_err(fgb_to_geozero_err)?;
                let mut ds = if let Some(bbox) = &args.extent {
                    ds.select_bbox(bbox.minx, bbox.miny, bbox.maxx, bbox.maxy)
                        .map_err(fgb_to_geozero_err)?
                } else {
                    ds.select_all().map_err(fgb_to_geozero_err)?
                };
                ds.process_features(processor)
            }
            Some("wkt") => GeozeroDatasource::process(&mut WktReader(&mut filein), processor),
            _ => panic!("Unknown input file extension"),
        }
    }
}

async fn process(args: Cli) -> Result<()> {
    let mut fout = BufWriter::new(File::create(&args.dest)?);
    match args.dest.extension().and_then(OsStr::to_str) {
        Some("csv") => transform(&args, &mut CsvWriter::new(&mut fout)).await?,
        Some("wkt") => transform(&args, &mut WktWriter::new(&mut fout)).await?,
        Some("json") | Some("geojson") => {
            transform(&args, &mut GeoJsonWriter::new(&mut fout)).await?
        }
        Some("fgb") => {
            let mut fgb =
                FgbWriter::create("fgb", GeometryType::Unknown).map_err(fgb_to_geozero_err)?;
            transform(&args, &mut fgb).await?;
            fgb.write(&mut fout).map_err(fgb_to_geozero_err)?;
        }
        Some("svg") => {
            let extent = get_extent(&args).await?;
            let mut processor = SvgWriter::new(&mut fout, true);
            // TODO: get width/height from args
            processor.set_style(args.svg_style.clone());
            processor.set_dimensions(extent.minx, extent.miny, extent.maxx, extent.maxy, 800, 600);
            transform(&args, &mut processor).await?;
        }
        _ => panic!("Unknown output file extension"),
    }
    Ok(())
}

async fn get_extent(args: &Cli) -> Result<Extent> {
    match args.extent {
        Some(extent) => Ok(extent),
        None => {
            let mut bounds_processor = geozero::bounds::BoundsProcessor::new();
            transform(args, &mut bounds_processor).await?;
            let Some(computed_bounds) = bounds_processor.bounds() else {
                return Ok(Extent {
                    minx: 0.0,
                    miny: 0.0,
                    maxx: 0.0,
                    maxy: 0.0,
                });
            };

            Ok(Extent {
                minx: computed_bounds.min_x(),
                miny: computed_bounds.min_y(),
                maxx: computed_bounds.max_x(),
                maxy: computed_bounds.max_y(),
            })
        }
    }
}

fn fgb_to_geozero_err(fgb_err: flatgeobuf::Error) -> GeozeroError {
    match fgb_err {
        flatgeobuf::Error::MissingMagicBytes => {
            GeozeroError::Dataset("Malformed FGB - missing Magic Bytes".to_string())
        }
        flatgeobuf::Error::NoIndex => GeozeroError::Dataset(
            "No Index: Index operations are not supported for this FGB".to_string(),
        ),
        flatgeobuf::Error::HttpClient(e) => GeozeroError::HttpError(e.to_string()),
        flatgeobuf::Error::IllegalHeaderSize(size) => {
            GeozeroError::Dataset(format!("Malformed FGB - Illegal header size: {size}"))
        }
        flatgeobuf::Error::InvalidFlatbuffer(e) => {
            GeozeroError::Dataset(format!("Invalid Flatbuffer: {e}"))
        }
        flatgeobuf::Error::IO(io) => GeozeroError::IoError(io),
        flatgeobuf::Error::UnsupportedGeometryType(error_message) => {
            GeozeroError::Dataset(error_message)
        }
    }
}

#[tokio::main]
async fn main() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env).init();

    let args = Cli::parse();

    let result = process(args).await;

    if let Err(msg) = result {
        println!("Processing failed: {msg}");
        exit(1)
    }
}
