extern crate mozjpeg;
use mozjpeg::{Compress, ColorSpace, ScanMode};

use std::io::{Read, Write};

pub struct JpegImage {
    pub image: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl JpegImage {
    pub fn new(image: Vec<u8>, width: usize, height: usize) -> Self {
        Self {
            image,
            width,
            height,
        }
    }

    pub fn open(path: &str) -> Result<Self, String> {
        let mut raw_data = std::fs::File::open(path).map_err(|_| "Failed to open file".to_string())?;
        let mut buf = Vec::new();
        raw_data.read_to_end(&mut buf).map_err(|_| "Failed to read file".to_string())?;

        let image = image::load_from_memory(&buf).map_err(|_| "Failed to open image".to_string())?;
        let (width, height) = (image.width() as usize, image.height() as usize);

        Ok(Self {
            image: image.into_bytes(),
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
