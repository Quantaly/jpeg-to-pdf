use std::fmt::{self, Display, Formatter};

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
    ImageSections(img_parts::Error),
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
            ImageSections(e) => f.write_fmt(format_args!("failed to read image sections: {}", e)),
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

impl From<img_parts::Error> for Cause {
    fn from(e: img_parts::Error) -> Cause {
        Cause::ImageSections(e)
    }
}
