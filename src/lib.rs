#![doc(html_root_url = "https://docs.rs/jpeg-to-pdf/0.1.0")]
//! Creates PDFs from JPEG images.
//!
//! Images are embedded directly in the PDF, without any re-encoding.
//!
//! # Example
//!
//! ```no_run
//! use std::fs::{self, File};
//! use std::io::BufWriter;
//!
//! use jpeg_to_pdf::create_pdf_from_jpegs;
//! 
//! let one = fs::read("one.jpg").unwrap();
//! let two = fs::read("two.jpg").unwrap();
//! let three = fs::read("three.jpg").unwrap();
//!
//! let out_file = File::create("out.pdf").unwrap();
//! create_pdf_from_jpegs(vec![one, two, three], &mut BufWriter::new(out_file), None).unwrap();
//! ```

use exif::{Field, In, Reader as ExifReader, Tag, Value};
use jpeg_decoder::{Decoder as JpegDecoder, PixelFormat};
use ori::Orientation;
use printpdf::*;
use std::fmt::{self, Display, Formatter};
use std::io::{prelude::*, BufWriter, Cursor};

#[macro_use]
extern crate lazy_static;

mod ori;

lazy_static! {
    static ref DEFAULT_ORIENTATION: Field = Field {
        tag: Tag::Orientation,
        ifd_num: In::PRIMARY,
        value: Value::Short(vec![1]),
    };
}

/// Creates a PDF file from the provided JPEG data.
///
/// PDF data is written to `out`.
///
/// `dpi` defaults to `300.0`.
pub fn create_pdf_from_jpegs(
    jpegs: Vec<Vec<u8>>,
    out: &mut BufWriter<impl Write>,
    dpi: Option<f64>,
) -> Result<(), Error> {
    let dpi = dpi.unwrap_or(300.0);

    let doc = PdfDocument::empty("");

    for (index, image) in jpegs.into_iter().enumerate() {
        if let Err(cause) = add_page(image, &doc, dpi) {
            return Err(Error { index, cause });
        }
    }

    doc.save(out).map_err(|e| Error {
        index: 0,
        cause: Cause::PdfWrite(e),
    })
}

fn add_page(image: Vec<u8>, doc: &PdfDocumentReference, dpi: f64) -> Result<(), Cause> {
    let mut decoder = JpegDecoder::new(Cursor::new(&image));
    decoder.read_info()?;

    match decoder.info() {
        None => Err(Cause::UnexpectedImageInfo),
        Some(info) => {
            let exif = ExifReader::new().read_from_container(&mut Cursor::new(&image))?;

            let ori = match &exif
                .get_field(Tag::Orientation, In::PRIMARY)
                .unwrap_or(&DEFAULT_ORIENTATION)
                .value
            {
                Value::Short(v) => *v.first().unwrap_or(&1),
                _ => 1,
            };

            let ori = Orientation {
                value: ori,
                width: info.width,
                height: info.height,
            };

            let (page, layer) = doc.add_page(
                Px(ori.display_width() as usize).into_pt(dpi).into(),
                Px(ori.display_height() as usize).into_pt(dpi).into(),
                "",
            );

            let image = Image::from(ImageXObject {
                width: Px(info.width as usize),
                height: Px(info.height as usize),
                color_space: match info.pixel_format {
                    PixelFormat::L8 => ColorSpace::Greyscale,
                    PixelFormat::RGB24 => ColorSpace::Rgb,
                    PixelFormat::CMYK32 => ColorSpace::Cmyk,
                },
                bits_per_component: ColorBits::Bit8,
                interpolate: false,
                image_data: image,
                image_filter: Some(ImageFilter::DCT),
                clipping_bbox: None,
            });

            image.add_to_layer(
                doc.get_page(page).get_layer(layer),
                Some(Px(ori.translate_x() as usize).into_pt(dpi).into()),
                Some(Px(ori.translate_y() as usize).into_pt(dpi).into()),
                Some(ori.rotate_cw()),
                Some(ori.scale_x()),
                None,
                Some(dpi),
            );

            Ok(())
        }
    }
}

/// An error that might occur while creating a PDF from JPEGs.
#[derive(Debug)]
pub struct Error {
    pub index: usize,
    pub cause: Cause,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.cause {
            Cause::PdfWrite(_) => self.cause.fmt(f),
            _ => f.write_fmt(format_args!(
                "error with JPEG index {}: {}",
                self.index, self.cause
            )),
        }
    }
}

impl std::error::Error for Error {}

/// Things that might go wrong while creating a PDF from JPEGs.
#[derive(Debug)]
pub enum Cause {
    ImageInfo(jpeg_decoder::Error),
    UnexpectedImageInfo,
    ExifMetadata(exif::Error),
    PdfWrite(printpdf::errors::Error),
}

impl Display for Cause {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Cause::*;
        match self {
            ImageInfo(e) => f.write_fmt(format_args!("failed to read image info: {}", e)),
            UnexpectedImageInfo => {
                f.write_fmt(format_args!("unexpectedly failed to read image info"))
            }
            ExifMetadata(e) => f.write_fmt(format_args!("failed to read EXIF metadata: {}", e)),
            PdfWrite(e) => f.write_fmt(format_args!("failed to write PDF: {}", e)),
        }
    }
}

impl std::error::Error for Cause {}

impl From<jpeg_decoder::Error> for Cause {
    fn from(e: jpeg_decoder::Error) -> Cause {
        Cause::ImageInfo(e)
    }
}

impl From<exif::Error> for Cause {
    fn from(e: exif::Error) -> Cause {
        Cause::ExifMetadata(e)
    }
}
