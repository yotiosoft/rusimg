use std::path::PathBuf;
use rusimg::RusimgError;
use rusimg::*;
use image::DynamicImage;

mod rusimg;

/// Open an image file.
pub fn open_image(path: &str) -> Result<RusImg, RusimgError> {
    let path_buf = PathBuf::from(path);
    let img = rusimg::do_open_image(&path_buf)?;
    Ok(img)
}
