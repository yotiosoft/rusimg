use std::path::{Path, PathBuf};
use std::fs;
use std::fmt;
use glob::glob;
use parse::ArgStruct;
use rusimg::RusimgError;
use colored::*;
use rusimg::*;
use image::DynamicImage;

mod parse;
mod rusimg;

pub fn open_image(path: &PathBuf) -> Result<Img, RusimgError> {
    let img = rusimg::open_image(path)?;
    Ok(img)
}

pub fn get_dynamic_image(img: &Img) -> Result<DynamicImage, RusimgError> {
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

pub fn save_image(img: &mut Img, path: Option<&PathBuf>) -> Result<(), RusimgError> {
    _ = rusimg::save_image(path, &mut img.data, &img.extension, FileOverwriteAsk::YesToAll)?;
    Ok(())
}
