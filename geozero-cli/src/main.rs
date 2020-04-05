use flatgeobuf;
use geozero_api::{HttpReader, OpenOpts, Reader, SelectOpts};
use geozero_core::geojson::GeoJsonWriter;
use std::fs::File;
use std::io::BufWriter;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    /// The path or URL to the FlatGeobuf file to read
    input: String,
    /// The path to the file to write
    #[structopt(parse(from_os_str))]
    dest: std::path::PathBuf,
}

fn process(args: Cli) -> Result<(), std::io::Error> {
    let open_opts = OpenOpts {};
    let mut driver = flatgeobuf::Driver::open(args.input, &open_opts)?;

    let select_opts = SelectOpts { bbox: None };
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

    let select_opts = SelectOpts { bbox: None };
    driver.select(&select_opts).await;

    let mut fout = BufWriter::new(File::create(args.dest)?);
    let mut json = GeoJsonWriter::new(&mut fout);
    driver.process(&mut json).await;
    Ok(())
}

fn main() {
    let args = Cli::from_args();

    if args.input.starts_with("http") {
        let _ = process_url(args);
    } else {
        let _ = process(args);
    }
}
