extern crate oxipng;

use std::io::{Read, Write};
use std::fs::Metadata;
use image::DynamicImage;

use crate::rusimg::Rusimg;

pub struct PngImage {
    pub image_into_bytes: Vec<u8>,
    pub binary_data: Vec<u8>,
    pub image: DynamicImage,
    pub width: usize,
    pub height: usize,
    pub metadata_input: Metadata,
    pub metadata_output: Option<Metadata>,
    pub filepath_input: String,
    pub filepath_output: Option<String>,
}

impl Rusimg for PngImage {
    fn new(image: DynamicImage, source_path: String, source_metadata: Metadata) -> Result<Self, String> {
        let (width, height) = (image.width() as usize, image.height() as usize);

        Ok(Self {
            image_into_bytes: image.clone().into_bytes(),
            binary_data: Vec::new(),
            image,
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
            image_into_bytes: image.clone().into_bytes(),
            binary_data: buf,
            image,
            width,
            height,
            metadata_input,
            metadata_output: None,
            filepath_input: path.to_string(),
            filepath_output: None,
        })
    }

    fn save(&mut self, path: &Option<String>) -> Result<(), String> {
        let (mut file, save_path) = if let Some(path) = path {
            (std::fs::File::create(path).map_err(|_| "Failed to create file".to_string())?, path.to_string())
        }
        else {
            let path = format!("{}.{}", self.filepath_input, "png");
            (std::fs::File::create(&path).map_err(|_| "Failed to create file".to_string())?, path)
        };
        file.write_all(&self.image_into_bytes).map_err(|_| "Failed to write file".to_string())?;

        self.metadata_output = Some(file.metadata().map_err(|_| "Failed to get metadata".to_string())?);
        self.filepath_output = Some(save_path);

        Ok(())
    }

    fn compress(&mut self) -> Result<(), String> {
        match oxipng::optimize_from_memory(&self.binary_data, &oxipng::Options::default()) {
            Ok(data) => {
                self.image_into_bytes = data;
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
