extern crate image;
extern crate webp;

use image::{DynamicImage, EncodableLayout};

use std::fs::Metadata;
use std::io::{Read, Write};
use std::path::Path;

use crate::rusimg::Rusimg;

#[derive(Debug, Clone)]
pub struct WebpImage {
    pub image: DynamicImage,
    image_bytes: Option<Vec<u8>>,
    width: usize,
    height: usize,
    operations_count: u32,
    required_quality: Option<f32>,
    pub metadata_input: Metadata,
    pub metadata_output: Option<Metadata>,
    pub filepath_input: String,
    pub filepath_output: Option<String>,
}

impl Rusimg for WebpImage {
    fn import(image: DynamicImage, source_path: String, source_metadata: Metadata) -> Result<Self, String> {
        let (width, height) = (image.width() as usize, image.height() as usize);

        Ok(Self {
            image,
            image_bytes: None,
            width,
            height,
            operations_count: 0,
            required_quality: None,
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

        let webp_decoder = webp::Decoder::new(&buf).decode();
        if let Some(webp_decoder) = webp_decoder {
            let image = webp_decoder.to_image();
            let (width, height) = (image.width() as usize, image.height() as usize);

            Ok(Self {
                image,
                image_bytes: Some(buf),
                width,
                height,
                operations_count: 0,
                required_quality: None,
                metadata_input,
                metadata_output: None,
                filepath_input: path.to_string(),
                filepath_output: None,
            })
        }
        else {
            return Err("Failed to decode webp".to_string());
        }
    }

    fn save(&mut self, path: Option<&String>) -> Result<(), String> {
        let save_path = Self::save_filepath(&self.filepath_input, path, &"webp".to_string());

        // 元が webp かつ操作回数が 0 なら encode しない
        let source_is_webp = Path::new(&self.filepath_input).extension().and_then(|s| s.to_str()).unwrap_or("").to_string() == "webp";
        if source_is_webp && self.operations_count == 0 && self.image_bytes.is_some() {
            let mut file = std::fs::File::create(&save_path).map_err(|_| "Failed to create file".to_string())?;
            file.write_all(self.image_bytes.as_ref().unwrap()).map_err(|_| "Failed to write file".to_string())?;

            self.metadata_output = Some(file.metadata().map_err(|_| "Failed to get metadata".to_string())?);
            self.filepath_output = Some(save_path);

            return Ok(());
        }

        // quality
        let quality = if let Some(q) = self.required_quality {
            q       // 指定されていればその値
        }
        else if source_is_webp {
            100.0   // 元が webp なら 既定で 100.0
        }
        else {
            75.0    // それ以外なら 75.0
        };
       
        // DynamicImage を 保存
        let encoded_webp = webp::Encoder::from_image(&self.image).map_err(|e| format!("Failed to encode webp: {}", e))?.encode(quality);

        let mut file = std::fs::File::create(&save_path).map_err(|_| "Failed to create file".to_string())?;
        file.write_all(&encoded_webp.as_bytes()).map_err(|_| "Failed to write file".to_string())?;

        self.metadata_output = Some(file.metadata().map_err(|_| "Failed to get metadata".to_string())?);
        self.filepath_output = Some(save_path);

        Ok(())
    }

    fn compress(&mut self, quality: Option<f32>) -> Result<(), String> {
        // webp の場合、圧縮は save() で行う
        self.required_quality = quality;
        self.operations_count += 1;
        Ok(())
    }
}
