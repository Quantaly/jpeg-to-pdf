use std::fs::{self, File};
use std::io::{self, BufWriter};
use std::path::PathBuf;
use std::process;

use jpeg_to_pdf::create_pdf_from_jpegs;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(parse(from_os_str))]
    images: Vec<PathBuf>,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let out_file = File::create(opt.output)?;
    let mut jpegs = Vec::with_capacity(opt.images.len());
    for image in opt.images {
        jpegs.push(fs::read(image)?);
    }

    if let Err(e) = create_pdf_from_jpegs(jpegs, &mut BufWriter::new(out_file), None) {
        eprintln!("{}", e);
        process::exit(-1);
    }

    Ok(())
}
