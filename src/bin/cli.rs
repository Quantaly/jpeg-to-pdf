use std::fs::{self, File};
use std::io::{self, prelude::*, BufWriter};
use std::path::PathBuf;
use std::process;

use jpeg_to_pdf::JpegToPdf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// By default, uses the same name as the first input, with the extension changed to ".pdf"
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(parse(from_os_str))]
    images: Vec<PathBuf>,

    #[structopt(long, default_value = "300")]
    dpi: f64,

    /// Strip EXIF metadata from the embedded images
    #[structopt(long)]
    strip_exif: bool,

    /// Add a title to the generated PDF
    #[structopt(long)]
    title: Option<String>,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    if opt.images.is_empty() {
        eprintln!("At least one image must be provided");
        process::exit(-1);
    }

    let out_file = File::create(match opt.output {
        Some(p) => p,
        None => {
            let mut out = opt.images[0].clone();
            out.set_extension("pdf");
            out
        }
    })?;

    let mut job = JpegToPdf::new();
    for image in opt.images {
        // have to do this with a for loop instead of job.add_images() to use the ? error-handling operator
        job = job.add_image(fs::read(image)?);
    }
    job = job
        .set_dpi(opt.dpi)
        .strip_exif(opt.strip_exif)
        .set_document_title(opt.title.unwrap_or_else(String::new));

    let mut out = BufWriter::new(out_file);
    if let Err(e) = job.create_pdf(&mut out) {
        eprintln!("{}", e);
        process::exit(-1);
    }
    out.flush()?;
    Ok(())
}
