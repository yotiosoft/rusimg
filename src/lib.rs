use std::path::PathBuf;
use rusimg::RusimgError;
use rusimg::*;
use image::DynamicImage;

mod parse;
mod rusimg;

/// Open an image file.
pub fn open_image(path: &PathBuf) -> Result<RusImg, RusimgError> {
    let img = rusimg::open_image(path)?;
    Ok(img)
}

/// Resize an image.
/// It must be called after open_image().
/// Set ratio to 100 to keep the original size.
pub fn resize(img: &mut RusImg, ratio: u8) -> Result<(), RusimgError> {
    rusimg::resize(img, ratio)?;
    Ok(())
}

/// Trim an image.
/// It must be called after open_image().
pub fn trim(img: &mut RusImg, trim_x: u32, trim_y: u32, trim_w: u32, trim_h: u32) -> Result<(), RusimgError> {
    rusimg::trim(img, (trim_x, trim_y), (trim_w, trim_h))?;
    Ok(())
}

/// Grayscale an image.
/// It must be called after open_image().
pub fn grayscale(img: &mut RusImg) -> Result<(), RusimgError> {
    rusimg::grayscale(img)?;
    Ok(())
}

/// Compress an image.
/// It must be called after open_image().
/// Set quality to 100 to keep the original quality.
pub fn compress(img: &mut RusImg, quality: Option<f32>) -> Result<(), RusimgError> {
    rusimg::compress(&mut img.data, &img.extension, quality)?;
    Ok(())
}

/// Convert an image to another format.
/// It must be called after open_image().
pub fn convert(img: &mut RusImg, new_extension: Extension) -> Result<(), RusimgError> {
    rusimg::convert(img, &new_extension)?;
    Ok(())
}

/// View an image on the terminal.
/// It must be called after open_image().
pub fn view(img: &mut RusImg) -> Result<(), RusimgError> {
    rusimg::view(img)?;
    Ok(())
}

/// Get a DynamicImage from an Img.
pub fn get_dynamic_image(img: &RusImg) -> Result<DynamicImage, RusimgError> {
    let dynamic_image = match img.extension {
        Extension::Png => {
            if img.data.png.is_none() {
                return Err(RusimgError::FailedToGetDynamicImage);
            }
            img.data.png.as_ref().unwrap().image.clone()
        }
        Extension::Jpeg => {
            if img.data.jpeg.is_none() {
                return Err(RusimgError::FailedToGetDynamicImage);
            }
            img.data.jpeg.as_ref().unwrap().image.clone()
        }
        Extension::Bmp => {
            if img.data.bmp.is_none() {
                return Err(RusimgError::FailedToGetDynamicImage);
            }
            img.data.bmp.as_ref().unwrap().image.clone()
        }
        Extension::Webp => {
            if img.data.webp.is_none() {
                return Err(RusimgError::FailedToGetDynamicImage);
            }
            img.data.webp.as_ref().unwrap().image.clone()
        }
    };
    Ok(dynamic_image)
}

pub fn save_image(img: &mut RusImg, path: Option<&PathBuf>) -> Result<(), RusimgError> {
    _ = rusimg::save_image(path, &mut img.data, &img.extension, FileOverwriteAsk::YesToAll)?;
    Ok(())
}
