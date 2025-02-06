use clap::Parser;
use flatgeobuf::{FgbReader, FgbWriter, GeometryType, HttpFgbReader};
use geo::Rect;
use geoarrow::io::parquet::{GeoParquetReaderOptions, GeoParquetRecordBatchReaderBuilder};
use geoarrow::io::RecordBatchReader;
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

    /// The path to the input file, or the URL for remote FlatGeobuf files
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

async fn transform<P: FeatureProcessor>(args: Cli, processor: &mut P) -> Result<()> {
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
                    .expect("must specify --csv-geometry-column=<column name> when parsing CSV");
                let mut ds = CsvReader::new(&geometry_column_name, &mut filein);
                GeozeroDatasource::process(&mut ds, processor)
            }
            Some("json") | Some("geojson") => {
                GeozeroDatasource::process(&mut GeoJsonReader(filein), processor)
            }
            Some("jsonl") | Some("geojsonl") => {
                GeozeroDatasource::process(&mut GeoJsonLineReader::new(filein), processor)
            }
            Some("parquet") | Some("geoparquet") => {
                let mut geo_options = GeoParquetReaderOptions::default();
                if let Some(bbox) = &args.extent {
                    geo_options = geo_options.with_bbox(
                        Rect::new((bbox.minx, bbox.miny), (bbox.maxx, bbox.maxy)),
                        None,
                    );
                }
                let reader = GeoParquetRecordBatchReaderBuilder::try_new_with_options(
                    File::open(path_in)?,
                    Default::default(),
                    geo_options,
                )
                .map_err(arrow_to_geozero_err)?
                .build()
                .map_err(arrow_to_geozero_err)?;

                let mut wrapper = RecordBatchReader::new(Box::new(reader));
                wrapper.process(processor)
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
        Some("csv") => transform(args, &mut CsvWriter::new(&mut fout)).await?,
        Some("wkt") => transform(args, &mut WktWriter::new(&mut fout)).await?,
        Some("json") | Some("geojson") => {
            transform(args, &mut GeoJsonWriter::new(&mut fout)).await?
        }
        Some("fgb") => {
            let mut fgb =
                FgbWriter::create("fgb", GeometryType::Unknown).map_err(fgb_to_geozero_err)?;
            transform(args, &mut fgb).await?;
            fgb.write(&mut fout).map_err(fgb_to_geozero_err)?;
        }
        Some("svg") => {
            let mut processor = SvgWriter::new(&mut fout, true);
            set_dimensions(&mut processor, args.extent);
            transform(args, &mut processor).await?;
        }
        _ => panic!("Unknown output file extension"),
    }
    Ok(())
}

fn set_dimensions(processor: &mut SvgWriter<&mut BufWriter<File>>, extent: Option<Extent>) {
    if let Some(extent) = extent {
        processor.set_dimensions(extent.minx, extent.miny, extent.maxx, extent.maxy, 800, 600);
    } else {
        // TODO: get image size as opts and full extent from data
        processor.set_dimensions(-180.0, -90.0, 180.0, 90.0, 800, 600);
    }
}

fn arrow_to_geozero_err(parquet_err: geoarrow::error::GeoArrowError) -> GeozeroError {
    match parquet_err {
        geoarrow::error::GeoArrowError::IOError(e) => GeozeroError::IoError(e),
        err => GeozeroError::Dataset(format!("Unknown GeoArrow error: {err:?}")),
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
