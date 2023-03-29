use std::io::{Read, Write, Cursor};
use std::fs::Metadata;
use image::DynamicImage;

use crate::rusimg::Rusimg;

fn compress(mut image: &DynamicImage, mut binary_data: &Vec<u8>, quality: Option<f32>) -> Result<Vec<u8>, String> {
    // quality の値に応じて level を設定
    let level = if let Some(q) = quality {
        if q <= 17.0 {
            1
        }
        else if q > 17.0 && q <= 34.0 {
            2
        }
        else if q > 34.0 && q <= 51.0 {
            3
        }
        else if q > 51.0 && q <= 68.0 {
            4
        }
        else if q > 68.0 && q <= 85.0 {
            5
        }
        else {
            6
        }
    }
    else {
        4       // default
    };

    match oxipng::optimize_from_memory(&binary_data, &oxipng::Options::from_preset(level)) {
        Ok(data) => {
            println!("Compress: Done.");
            Ok(data)
        },
        Err(e) => match e {
            oxipng::PngError::DeflatedDataTooLong(s) => Err(format!("deflated data too long: {}", s)),
            oxipng::PngError::TimedOut => Err("timed out".to_string()),
            oxipng::PngError::NotPNG => Err("not png".to_string()),
            oxipng::PngError::APNGNotSupported => Err("apng not supported".to_string()),
            oxipng::PngError::InvalidData => Err("invalid data".to_string()),
            oxipng::PngError::TruncatedData => Err("truncated data".to_string()),
            oxipng::PngError::ChunkMissing(s) => Err(format!("chunk missing: {}", s)),
            oxipng::PngError::Other(s) => Err(format!("other: {}", s)),
            _ => Err("unknown error".to_string()),
        }
    }
}
