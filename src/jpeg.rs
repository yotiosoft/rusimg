extern crate mozjpeg;
use mozjpeg::{Compress, ColorSpace, ScanMode};

use std::fs::Metadata;
use std::io::{Read, Write};
use std::path::Path;

pub struct JpegImage {
    pub image: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub extension_str: String,
    pub metadata_input: Metadata,
    pub metadata_output: Option<Metadata>,
    pub filepath_input: String,
    pub filepath_output: Option<String>,
}

impl JpegImage {
    pub fn open(path: &str) -> Result<Self, String> {
        let mut raw_data = std::fs::File::open(path).map_err(|_| "Failed to open file".to_string())?;
        let mut buf = Vec::new();
        raw_data.read_to_end(&mut buf).map_err(|_| "Failed to read file".to_string())?;
        let metadata_input = raw_data.metadata().map_err(|_| "Failed to get metadata".to_string())?;

        let image = image::load_from_memory(&buf).map_err(|_| "Failed to open image".to_string())?;
        let (width, height) = (image.width() as usize, image.height() as usize);

        let extension_str = Path::new(path).extension().and_then(|s| s.to_str()).unwrap_or("").to_string();

        Ok(Self {
            image: image.into_bytes(),
            width,
            height,
            extension_str,
            metadata_input,
            metadata_output: None,
            filepath_input: path.to_string(),
            filepath_output: None,
        })
    }

    pub fn save(&mut self, path: &Option<String>) -> Result<(), String> {
        let (mut file, save_path) = if let Some(path) = path {
            (std::fs::File::create(path).map_err(|_| "Failed to create file".to_string())?, path.to_string())
        }
        else {
            let path = format!("{}.{}", self.filepath_input, self.extension_str);
            (std::fs::File::create(&path).map_err(|_| "Failed to create file".to_string())?, path)
        };
        file.write_all(&self.image).map_err(|_| "Failed to write file".to_string())?;

        self.metadata_output = Some(file.metadata().map_err(|_| "Failed to get metadata".to_string())?);
        self.filepath_output = Some(save_path);

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
