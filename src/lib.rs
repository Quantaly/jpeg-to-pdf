#![doc(html_root_url = "https://docs.rs/jpeg-to-pdf/0.2.3")]
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
use exif::{In, Reader as ExifReader, Tag};
use img_parts::{jpeg::Jpeg, ImageEXIF};
use jpeg_decoder::{Decoder as JpegDecoder, PixelFormat};
use ori::Orientation;
use printpdf::*;
use std::io::{prelude::*, BufWriter, Cursor};

mod errors;
mod ori;

mod tests;
/// Creates a PDF from JPEG images.
pub struct JpegToPdf {
    images: Vec<Vec<u8>>,
    dpi: f64,
    strip_exif: bool,
    document_title: String,
    creation_date: OffsetDateTime,
    mod_date: OffsetDateTime,
}

impl JpegToPdf {
    pub fn new() -> JpegToPdf {
        JpegToPdf {
            images: Vec::new(),
            dpi: 300.0,
            strip_exif: false,
            document_title: String::new(),
            creation_date: OffsetDateTime::now_utc(),
            mod_date: OffsetDateTime::now_utc(),
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

    /// Sets the creation date of the PDF output.
    pub fn set_creation_date(mut self, creation_date: OffsetDateTime) -> JpegToPdf {
        self.creation_date = creation_date;
        self
    }

    /// Sets the modification date of the PDF output.
    pub fn set_mod_date(mut self, mod_date: OffsetDateTime) -> JpegToPdf {
        self.mod_date = mod_date;
        self
    }

    /// Writes the PDF output to `out`.
    pub fn create_pdf(self, out: &mut BufWriter<impl Write>) -> Result<(), Error> {
        let (dpi, strip_exif) = (self.dpi, self.strip_exif);

        let doc = PdfDocument::empty(self.document_title)
            .with_creation_date(self.creation_date)
            .with_mod_date(self.mod_date);

        self.images
            .into_iter()
            .enumerate()
            .try_for_each(|(index, image)| {
                add_page(image, &doc, dpi, strip_exif).map_err(|cause| Error { index, cause })
            })
            .and_then(|()| {
                doc.save(out).map_err(|e| Error {
                    index: 0,
                    cause: Cause::PdfWrite(e),
                })
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
        None => Err(Cause::UnexpectedImageInfo), // decoder.read_info would return Err, so we should never see this
        Some(info) => {
            let mut image = Jpeg::from_bytes(image.into())?;

            let ori = image
                .exif()
                .and_then(|exif_data| ExifReader::new().read_raw(exif_data.to_vec()).ok())
                .and_then(|exif| {
                    exif.get_field(Tag::Orientation, In::PRIMARY)
                        .and_then(|field| field.value.get_uint(0))
                })
                .unwrap_or(1);

            let ori = Orientation {
                value: ori,
                width: info.width as usize,
                height: info.height as usize,
            };

            if strip_exif {
                image.set_exif(None);
            }

            let mut image_data = Vec::new();
            image.encoder().write_to(&mut image_data).unwrap();

            let (page, layer) = doc.add_page(
                Px(ori.display_width()).into_pt(dpi).into(),
                Px(ori.display_height()).into_pt(dpi).into(),
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
                ori.translate_x().map(|px| Px(px).into_pt(dpi).into()),
                ori.translate_y().map(|px| Px(px).into_pt(dpi).into()),
                ori.rotate_cw(),
                ori.scale_x(),
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
