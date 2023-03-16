extern crate oxipng;
use oxipng::Deflaters;

use std::io::{Read, Write};

pub struct PngImage {
    pub image: Vec<u8>,
    pub raw_image: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl PngImage {
    pub fn new(image: Vec<u8>, raw_image: Vec<u8>, width: usize, height: usize) -> Self {
        Self {
            image,
            raw_image,
            width,
            height,
        }
    }

    pub fn open(path: &str) -> Result<Self, String> {
        let mut file = std::fs::File::open(path).map_err(|_| "Failed to open file".to_string())?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(|_| "Failed to read file".to_string())?;

        let image = image::load_from_memory(&buf).map_err(|_| "Failed to open image".to_string())?;
        let (width, height) = (image.width() as usize, image.height() as usize);

        Ok(Self {
            image: image.into_bytes(),
            raw_image: buf,
            width,
            height,
        })
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let mut file = std::fs::File::create(path).map_err(|_| "Failed to create file".to_string())?;
        file.write_all(&self.image).map_err(|_| "Failed to write file".to_string())?;

        Ok(())
    }

    pub fn compress(&mut self) -> Result<(), String> {
        println!("compressing png image...");
        let mut options = oxipng::Options::default();
        if let Deflaters::Libdeflater { compression } = &mut options.deflate {
            *compression = 5;
        }
        match oxipng::optimize_from_memory(&self.raw_image, &oxipng::Options::default()) {
            Ok(data) => {
                self.image = data;
                Ok(())
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
}
