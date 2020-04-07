use flatgeobuf;
use geozero_api::{Extent, HttpReader, OpenOpts, Reader, SelectOpts};
use geozero_core::geojson::GeoJsonWriter;
use std::fs::File;
use std::io::BufWriter;
use std::num::ParseFloatError;
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

fn parse_extent(src: &str) -> Result<Extent, ParseFloatError> {
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

fn process(args: Cli) -> Result<(), std::io::Error> {
    let open_opts = OpenOpts {};
    let mut driver = flatgeobuf::Driver::open(args.input, &open_opts)?;

    let select_opts = SelectOpts {
        extent: args.extent,
    };
    driver.select(&select_opts);

    let mut fout = BufWriter::new(File::create(args.dest)?);
    let mut json = GeoJsonWriter::new(&mut fout);
    driver.process(&mut json);
    Ok(())
}

#[tokio::main]
async fn process_url(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let open_opts = OpenOpts {};
    let mut driver = flatgeobuf::HttpDriver::open(args.input, &open_opts).await?;

    let select_opts = SelectOpts {
        extent: args.extent,
    };
    driver.select(&select_opts).await;

    let mut fout = BufWriter::new(File::create(args.dest)?);
    let mut json = GeoJsonWriter::new(&mut fout);
    driver.process(&mut json).await;
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
