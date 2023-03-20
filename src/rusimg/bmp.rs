use image::DynamicImage;

use std::fs::Metadata;
use std::io::Read;
use std::path::Path;

use crate::rusimg::Rusimg;

#[derive(Debug, Clone)]
pub struct BmpImage {
    pub image: DynamicImage,
    width: usize,
    height: usize,
    pub metadata_input: Metadata,
    pub metadata_output: Option<Metadata>,
    pub filepath_input: String,
    pub filepath_output: Option<String>,
}

impl Rusimg for BmpImage {
    fn import(image: DynamicImage, source_path: String, source_metadata: Metadata) -> Result<Self, String> {
        let (width, height) = (image.width() as usize, image.height() as usize);

        Ok(Self {
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
        let mut raw_data = std::fs::File::open(path).map_err(|_| "Failed to open file".to_string())?;
        let mut buf = Vec::new();
        raw_data.read_to_end(&mut buf).map_err(|_| "Failed to read file".to_string())?;
        let metadata_input = raw_data.metadata().map_err(|_| "Failed to get metadata".to_string())?;

        let image = image::load_from_memory(&buf).map_err(|_| "Failed to open image".to_string())?;
        let (width, height) = (image.width() as usize, image.height() as usize);

        let extension_str = Path::new(path).extension().and_then(|s| s.to_str()).unwrap_or("").to_string();

        Ok(Self {
            image,
            width,
            height,
            metadata_input,
            metadata_output: None,
            filepath_input: path.to_string(),
            filepath_output: None,
        })
    }

    fn save(&mut self, path: Option<&String>) -> Result<(), String> {
        let save_path = Self::save_filepath(&self.filepath_input, path, &"bmp".to_string());
        
        self.image.save(&save_path).map_err(|e| format!("Failed to save image: {}", e.to_string()))?;
        self.metadata_output = Some(std::fs::metadata(&save_path).map_err(|_| "Failed to get metadata".to_string())?);
        self.filepath_output = Some(save_path);

        Ok(())
    }

    fn compress(&mut self, quality: Option<f32>) -> Result<(), String> {
        Err("BMP images cannot be compressed".to_string())
    }
}
