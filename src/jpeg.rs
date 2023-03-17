extern crate mozjpeg;
use mozjpeg::{Compress, ColorSpace, ScanMode};

use std::io::{Read, Write};
use std::path::Path;

pub struct JpegImage {
    pub image: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub extension_str: String,
}

impl JpegImage {
    pub fn new(image: Vec<u8>, width: usize, height: usize, extension_str: String) -> Self {
        Self {
            image,
            width,
            height,
            extension_str,
        }
    }

    pub fn open(path: &str) -> Result<Self, String> {
        let mut raw_data = std::fs::File::open(path).map_err(|_| "Failed to open file".to_string())?;
        let mut buf = Vec::new();
        raw_data.read_to_end(&mut buf).map_err(|_| "Failed to read file".to_string())?;

        let image = image::load_from_memory(&buf).map_err(|_| "Failed to open image".to_string())?;
        let (width, height) = (image.width() as usize, image.height() as usize);

        let extension_str = Path::new(path).extension().and_then(|s| s.to_str()).unwrap_or("").to_string();

        Ok(Self {
            image: image.into_bytes(),
            width,
            height,
            extension_str,
        })
    }

    pub fn save(&self, path: &Option<String>) -> Result<(), String> {
        let mut file = if let Some(path) = path {
            std::fs::File::create(path).map_err(|_| "Failed to create file".to_string())?
        }
        else {
            std::fs::File::create(&format!("{}.{}", "output", self.extension_str)).map_err(|_| "Failed to create file".to_string())?
        };
        file.write_all(&self.image).map_err(|_| "Failed to write file".to_string())?;

        Ok(())
    }

    pub fn compress(&mut self) -> Result<(), String> {
        let mut compress = Compress::new(ColorSpace::JCS_RGB);
        compress.set_scan_optimization_mode(ScanMode::AllComponentsTogether);
        compress.set_size(self.width, self.height);
        compress.set_mem_dest();
        compress.start_compress();
        compress.write_scanlines(&self.image);
        compress.finish_compress();

        self.image = compress.data_to_vec().map_err(|_| "Failed to compress jpeg image".to_string())?;

        Ok(())
    }
}
