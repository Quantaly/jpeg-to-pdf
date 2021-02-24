#![doc(html_root_url = "https://docs.rs/jpeg-to-pdf/0.2.2")]
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
//! use jpeg_to_pdf::JpegToPdf;
//!
//! let out_file = File::create("out.pdf").unwrap();
//!
//! JpegToPdf::new()
//!     .add_image(fs::read("one.jpg").unwrap())
//!     .add_image(fs::read("two.jpg").unwrap())
//!     .add_image(fs::read("three.jpg").unwrap())
//!     .create_pdf(&mut BufWriter::new(out_file))
//!     .unwrap();
//! ```

use errors::Error;
pub use errors::*;
use exif::{Field, In, Reader as ExifReader, Tag, Value};
use img_parts::{jpeg::Jpeg, ImageEXIF};
use jpeg_decoder::{Decoder as JpegDecoder, PixelFormat};
use ori::Orientation;
use printpdf::*;
use std::io::{prelude::*, BufWriter, Cursor};

#[macro_use]
extern crate lazy_static;

mod errors;
mod ori;

mod tests;

lazy_static! {
    static ref DEFAULT_ORIENTATION: Field = Field {
        tag: Tag::Orientation,
        ifd_num: In::PRIMARY,
        value: Value::Short(vec![1]),
    };
}

/// Creates a PDF from JPEG images.
pub struct JpegToPdf {
    images: Vec<Vec<u8>>,
    dpi: f64,
    strip_exif: bool,
    document_title: String,
}

impl JpegToPdf {
    pub fn new() -> JpegToPdf {
        JpegToPdf {
            images: Vec::new(),
            dpi: 300.0,
            strip_exif: false,
            document_title: String::new(),
        }
    }

    /// Add an image to the PDF output.
    pub fn add_image(mut self, image: Vec<u8>) -> JpegToPdf {
        self.images.push(image);
        self
    }

    /// Add one or more images to the PDF output.
    pub fn add_images(mut self, images: impl IntoIterator<Item = Vec<u8>>) -> JpegToPdf {
        self.images.extend(images);
        self
    }

    /// Set the DPI scaling of the PDF output.
    pub fn set_dpi(mut self, dpi: f64) -> JpegToPdf {
        self.dpi = dpi;
        self
    }

    /// Strip EXIF metadata from the provided images.
    ///
    /// Some PDF renderers have issues rendering JPEG images that still have EXIF metadata.
    pub fn strip_exif(mut self, strip_exif: bool) -> JpegToPdf {
        self.strip_exif = strip_exif;
        self
    }

    /// Sets the title of the PDF output.
    pub fn set_document_title(mut self, document_title: impl Into<String>) -> JpegToPdf {
        self.document_title = document_title.into();
        self
    }

    /// Writes the PDF output to `out`.
    pub fn create_pdf(self, out: &mut BufWriter<impl Write>) -> Result<(), Error> {
        let doc = PdfDocument::empty(self.document_title);
        for (index, image) in self.images.into_iter().enumerate() {
            if let Err(cause) = add_page(image, &doc, self.dpi, self.strip_exif) {
                return Err(Error { index, cause });
            }
        }
        doc.save(out).map_err(|e| Error {
            index: 0,
            cause: Cause::PdfWrite(e),
        })
    }
}

fn add_page(
    image: Vec<u8>,
    doc: &PdfDocumentReference,
    dpi: f64,
    strip_exif: bool,
) -> Result<(), Cause> {
    let mut decoder = JpegDecoder::new(Cursor::new(&image));
    decoder.read_info()?;

    match decoder.info() {
        None => Err(Cause::UnexpectedImageInfo),
        Some(info) => {
            let mut image = Jpeg::from_bytes(image.into())?;

            let ori = match image.exif() {
                None => 1,
                Some(exif) => match ExifReader::new().read_raw(exif.to_vec()) {
                    Err(_) => 1,
                    Ok(exif) => match &exif
                        .get_field(Tag::Orientation, In::PRIMARY)
                        .unwrap_or(&DEFAULT_ORIENTATION)
                        .value
                    {
                        Value::Short(v) => *v.first().unwrap_or(&1),
                        _ => 1,
                    },
                },
            };

            let ori = Orientation {
                value: ori,
                width: info.width,
                height: info.height,
            };

            if strip_exif {
                image.set_exif(None);
            }

            let mut image_data = Vec::new();
            image.encoder().write_to(&mut image_data).unwrap();

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
                image_data,
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

/// Creates a PDF file from the provided JPEG data.
///
/// PDF data is written to `out`.
///
/// `dpi` defaults to `300.0`.
///
/// Please use [`JpegToPdf`] instead.
#[deprecated]
pub fn create_pdf_from_jpegs(
    jpegs: Vec<Vec<u8>>,
    out: &mut BufWriter<impl Write>,
    dpi: Option<f64>,
) -> Result<(), Error> {
    JpegToPdf::new()
        .add_images(jpegs)
        .set_dpi(dpi.unwrap_or(300.0))
        .create_pdf(out)
}
