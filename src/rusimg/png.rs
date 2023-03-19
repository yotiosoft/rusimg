extern crate oxipng;

use std::io::{Read, Write};
use std::fs::Metadata;
use image::DynamicImage;

use crate::rusimg::Rusimg;

#[derive(Debug, Clone)]
pub struct PngImage {
    binary_data: Vec<u8>,
    pub image: DynamicImage,
    image_bytes: Option<Vec<u8>>,
    width: usize,
    height: usize,
    pub metadata_input: Metadata,
    pub metadata_output: Option<Metadata>,
    pub filepath_input: String,
    pub filepath_output: Option<String>,
}

impl Rusimg for PngImage {
    fn new(image: DynamicImage, source_path: String, source_metadata: Metadata) -> Result<Self, String> {
        let (width, height) = (image.width() as usize, image.height() as usize);

        Ok(Self {
            binary_data: Vec::new(),
            image,
            image_bytes: None,
            width,
            height,
            metadata_input: source_metadata,
            metadata_output: None,
            filepath_input: source_path,
            filepath_output: None,
        })
    }

    fn open(path: &str) -> Result<Self, String> {
        let mut file = std::fs::File::open(path).map_err(|_| "Failed to open file".to_string())?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(|_| "Failed to read file".to_string())?;
        let metadata_input = file.metadata().map_err(|_| "Failed to get metadata".to_string())?;

        let image = image::load_from_memory(&buf).map_err(|_| "Failed to open image".to_string())?;
        let (width, height) = (image.width() as usize, image.height() as usize);

        Ok(Self {
            binary_data: buf,
            image,
            image_bytes: None,
            width,
            height,
            metadata_input,
            metadata_output: None,
            filepath_input: path.to_string(),
            filepath_output: None,
        })
    }

    fn save(&mut self, path: Option<&String>) -> Result<(), String> {
        let save_path = if let Some(path) = path {
            path.to_string()
        }
        else {
            format!("{}.{}", self.filepath_input, "png")
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
        match oxipng::optimize_from_memory(&self.binary_data, &oxipng::Options::default()) {
            Ok(data) => {
                self.image_bytes = Some(data);
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
