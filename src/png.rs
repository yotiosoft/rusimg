extern crate imagequant;

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
        let mut liq = imagequant::new();
        liq.set_speed(5).map_err(|_| "Failed to set speed".to_string())?;
        liq.set_quality(70, 99).map_err(|_| "Failed to set quality".to_string())?;

        // Describe the image

        let mut res = match liq.quantize(&self.raw_image, self.width, self.height, 0.0) {
            Ok(res) => res,
            Err(_) => return Err("Failed to quantize image".to_string()),
        };
    }
}
