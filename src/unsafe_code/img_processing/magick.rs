use std::sync::{Once, ONCE_INIT};
use std::convert::From;

use unsafe_code::{UnsafeError, UnsafeErrorKind};

use magick_rust::{MagickWand, magick_wand_genesis};
use magick_rust::bindings::ColorspaceType;

static INIT_WAND: Once = ONCE_INIT;

impl From<&'static str> for UnsafeError {
    fn from(err: &'static str) -> UnsafeError {
        UnsafeError::new(UnsafeErrorKind::ImageMagickError(err))
    }
}

pub fn convert_colorspace(pic: Vec<u8>) -> Result<Vec<u8>, UnsafeError> {
    INIT_WAND.call_once(|| {
        magick_wand_genesis();
    });

    let mut wand = MagickWand::new();
    match wand.read_image_blob(&pic) {
        Ok(()) => {
            let _ = wand.set_colorspace(ColorspaceType::RGBColorspace);
            match wand.write_image_blob("jpeg") {
                Ok(v) => Ok(v),
                Err(e) => Err(UnsafeError::from(e)),
            }
        },
        Err(e) => Err(UnsafeError::from(e))
    }
}