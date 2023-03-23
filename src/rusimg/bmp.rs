use image::DynamicImage;

use std::fs::Metadata;
use std::io::Read;

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

    fn compress(&mut self, _quality: Option<f32>) -> Result<(), String> {
        Err("BMP images cannot be compressed".to_string())
    }

    fn resize(&mut self, resize_ratio: u8) -> Result<(), String> {
        let nwidth = (self.width as f32 * (resize_ratio as f32 / 100.0)) as usize;
        let nheight = (self.height as f32 * (resize_ratio as f32 / 100.0)) as usize;
        
        self.image = self.image.resize(nwidth as u32, nheight as u32, image::imageops::FilterType::Lanczos3);

        println!("Resize: {}x{} -> {}x{}", self.width, self.height, nwidth, nheight);

        self.width = nwidth;
        self.height = nheight;

        Ok(())
    }

    fn trim(&mut self, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<(), String> {
        let mut w = trim_wh.0;
        let mut h = trim_wh.1;
        if self.width < (trim_xy.0 + w) as usize || self.height < (trim_xy.1 + h) as usize {
            if self.width > trim_xy.0 as usize || self.height > trim_xy.1 as usize {
                w = std::cmp::min(self.width as u32, w);
                h = std::cmp::min(self.height as u32, h);
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

        Ok(())
    }

    fn grayscale(&mut self) {
        self.image = self.image.grayscale();
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
