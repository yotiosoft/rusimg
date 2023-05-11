use image::{DynamicImage, EncodableLayout};

use std::fs::Metadata;
use std::io::{Read, Write};
use std::path::Path;

use crate::rusimg::Rusimg;
use crate::rusimg;

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
        else {
            75.0    // 既定: 75.0
        };
       
        // DynamicImage を （圧縮＆）保存
        let encoded_webp = webp::Encoder::from_image(&self.image).map_err(|e| format!("Failed to encode webp: {}", e))?.encode(quality);
        if self.required_quality.is_some() {
            println!("Compress: Done.");
        }

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

    fn resize(&mut self, resize_ratio: u8) -> Result<(), String> {
        let nwidth = (self.width as f32 * (resize_ratio as f32 / 100.0)) as usize;
        let nheight = (self.height as f32 * (resize_ratio as f32 / 100.0)) as usize;

        self.image = self.image.resize(nwidth as u32, nheight as u32, image::imageops::FilterType::Lanczos3);

        println!("Resize: {}x{} -> {}x{}", self.width, self.height, nwidth, nheight);

        self.width = nwidth;
        self.height = nheight;

        self.operations_count += 1;
        Ok(())
    }

    fn trim(&mut self, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<(), String> {
        let mut w = trim_wh.0;
        let mut h = trim_wh.1;
        if self.width < (trim_xy.0 + w) as usize || self.height < (trim_xy.1 + h) as usize {
            if self.width > trim_xy.0 as usize && self.height > trim_xy.1 as usize {
                w = if self.width < (trim_xy.0 + w) as usize { self.width as u32 - trim_xy.0 } else { trim_wh.0 };
                h = if self.height < (trim_xy.1 + h) as usize { self.height as u32 - trim_xy.1 } else { trim_wh.1 };
                println!("Required width or height is larger than image size. Corrected size: {}x{} -> {}x{}", trim_wh.0, trim_wh.1, w, h);
            }
            else {
                return Err(format!("Trim: Invalid trim point: {}x{}", trim_xy.0, trim_xy.1));
            }
        }

        self.image = self.image.crop(trim_xy.0, trim_xy.1, w, h);

        println!("Trim: {}x{} -> {}x{}", self.width, self.height, w, h);

        self.width = w as usize;
        self.height = h as usize;

        self.operations_count += 1;
        Ok(())
    }

    fn grayscale(&mut self) {
        self.image = self.image.grayscale();
        println!("Grayscale: Done.");
        self.operations_count += 1;
    }

    fn view(&self) -> Result<(), String> {
        let conf_width = self.width as f64 / std::cmp::max(self.width, self.height) as f64 * 100 as f64;
        let conf_height = self.height as f64 / std::cmp::max(self.width, self.height) as f64 as f64 * 50 as f64;
        let conf = viuer::Config {
            absolute_offset: false,
            width: Some(conf_width as u32),
            height: Some(conf_height as u32),    
            ..Default::default()
        };

        viuer::print(&self.image, &conf).map_err(|e| format!("Failed to view image: {}", e.to_string()))?;

        Ok(())
    }
}
