use flatgeobuf::*;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    /// The path to the FlatGeobuf file to read
    #[structopt(parse(from_os_str))]
    fgbpath: std::path::PathBuf,
    /// The path to the file to write
    #[structopt(parse(from_os_str))]
    dest: std::path::PathBuf,
}

fn main() -> std::result::Result<(), std::io::Error> {
    let args = Cli::from_args();

    let fin = File::open(args.fgbpath)?;
    let mut filein = BufReader::new(fin);
    let hreader = HeaderReader::read(&mut filein)?;
    let header = hreader.header();

    let mut freader = FeatureReader::select_all(&mut filein, &header)?;

    let mut fout = BufWriter::new(File::create(args.dest)?);
    freader.to_geojson(&mut filein, &header, &mut fout)
}
