extern crate mozjpeg;
use mozjpeg::{Compress, ColorSpace, ScanMode};
use image::DynamicImage;

use std::fs::Metadata;
use std::io::{Read, Write};
use std::path::Path;

use crate::rusimg::Rusimg;

pub struct JpegImage {
    pub image: DynamicImage,
    image_bytes: Option<Vec<u8>>,
    width: usize,
    height: usize,
    extension_str: String,
    pub metadata_input: Metadata,
    pub metadata_output: Option<Metadata>,
    pub filepath_input: String,
    pub filepath_output: Option<String>,
}

impl Rusimg for JpegImage {
    fn new(image: DynamicImage, source_path: String, source_metadata: Metadata) -> Result<Self, String> {
        let (width, height) = (image.width() as usize, image.height() as usize);

        Ok(Self {
            image,
            image_bytes: None,
            width,
            height,
            extension_str: "jpg".to_string(),
            metadata_input: source_metadata,
            metadata_output: None,
            filepath_input: source_path,
            filepath_output: None,
        })
    }

    fn open(path: &str) -> Result<Self, String> {
        let mut raw_data = std::fs::File::open(path).map_err(|_| "Failed to open file".to_string())?;
        let mut buf = Vec::new();
        raw_data.read_to_end(&mut buf).map_err(|_| "Failed to read file".to_string())?;
        let metadata_input = raw_data.metadata().map_err(|_| "Failed to get metadata".to_string())?;

        let image = image::load_from_memory(&buf).map_err(|_| "Failed to open image".to_string())?;
        let (width, height) = (image.width() as usize, image.height() as usize);

        let extension_str = Path::new(path).extension().and_then(|s| s.to_str()).unwrap_or("").to_string();

        Ok(Self {
            image,
            image_bytes: None,
            width,
            height,
            extension_str,
            metadata_input,
            metadata_output: None,
            filepath_input: path.to_string(),
            filepath_output: None,
        })
    }

    fn save(&mut self, path: &Option<String>) -> Result<(), String> {
        let save_path = if let Some(path) = path {
            path.to_string()
        }
        else {
            format!("{}.{}", self.filepath_input, self.extension_str)
        };
        
        // image_bytes == None の場合、DynamicImage を 保存
        if self.image_bytes.is_none() {
            self.image.save(&save_path).map_err(|e| format!("Failed to save image: {}", e.to_string()))?;
            self.metadata_output = Some(std::fs::metadata(&save_path).map_err(|_| "Failed to get metadata".to_string())?);
        }
        // image_bytes != None の場合、oxipng で圧縮したバイナリデータを保存
        else {
            let mut file = std::fs::File::create(&save_path).map_err(|_| "Failed to create file".to_string())?;
            file.write_all(&self.image_bytes.as_ref().unwrap()).map_err(|_| "Failed to write file".to_string())?;
            self.metadata_output = Some(file.metadata().map_err(|_| "Failed to get metadata".to_string())?);
        }

        self.filepath_output = Some(save_path);

        Ok(())
    }

    fn compress(&mut self) -> Result<(), String> {
        let image_bytes = self.image.clone().into_bytes();

        let mut compress = Compress::new(ColorSpace::JCS_RGB);
        compress.set_scan_optimization_mode(ScanMode::AllComponentsTogether);
        compress.set_size(self.width, self.height);
        compress.set_mem_dest();
        compress.start_compress();
        compress.write_scanlines(&image_bytes);
        compress.finish_compress();

        self.image_bytes = Some(compress.data_to_vec().map_err(|_| "Failed to compress image".to_string())?);

        Ok(())
    }
}
