use image::DynamicImage;

use std::fs::Metadata;
use std::path::PathBuf;

use crate::rusimg::Rusimg;
use super::{RusimgError, RusimgStatus};
use super::ImgSize;

#[derive(Debug, Clone)]
pub struct BmpImage {
    pub image: DynamicImage,
    size: ImgSize,
    pub metadata_input: Metadata,
    pub metadata_output: Option<Metadata>,
    pub filepath_input: PathBuf,
    pub filepath_output: Option<PathBuf>,
}

impl Rusimg for BmpImage {
    fn import(image: DynamicImage, source_path: PathBuf, source_metadata: Metadata) -> Result<Self, RusimgError> {
        let size = ImgSize { width: image.width() as usize, height: image.height() as usize };

        Ok(Self {
            image,
            size,
            metadata_input: source_metadata,
            metadata_output: None,
            filepath_input: source_path,
            filepath_output: None,
        })
    }

    fn open(path: PathBuf, image_buf: Vec<u8>, metadata: Metadata) -> Result<Self, RusimgError> {
        let image = image::load_from_memory(&image_buf).map_err(|e| RusimgError::FailedToOpenImage(e.to_string()))?;
        let size = ImgSize { width: image.width() as usize, height: image.height() as usize };

        Ok(Self {
            image,
            size,
            metadata_input: metadata,
            metadata_output: None,
            filepath_input: path,
            filepath_output: None,
        })
    }

    fn save(&mut self, path: Option<&PathBuf>) -> Result<RusimgStatus, RusimgError> {
        let save_path = Self::save_filepath(&self.filepath_input, path, &"bmp".to_string())?;
        // ファイルが存在するか？＆上書き確認
        if Self::check_file_exists(&save_path) == false {
            return Ok(RusimgStatus::Cancel);
        }
        
        self.image.to_rgba8().save(&save_path).map_err(|e| RusimgError::FailedToSaveImage(e.to_string()))?;
        self.metadata_output = Some(std::fs::metadata(&save_path).map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?);
        self.filepath_output = Some(save_path);

        Ok(RusimgStatus::Success)
    }

    fn compress(&mut self, _quality: Option<f32>) -> Result<(), RusimgError> {
        Err(RusimgError::BMPImagesCannotBeCompressed)
    }

    fn resize(&mut self, resize_ratio: u8) -> Result<(), RusimgError> {
        let nwidth = (self.size.width as f32 * (resize_ratio as f32 / 100.0)) as usize;
        let nheight = (self.size.height as f32 * (resize_ratio as f32 / 100.0)) as usize;
        
        self.image = self.image.resize(nwidth as u32, nheight as u32, image::imageops::FilterType::Lanczos3);

        println!("Resize: {}x{} -> {}x{}", self.size.width, self.size.height, nwidth, nheight);

        self.size.width = nwidth;
        self.size.height = nheight;

        Ok(())
    }

    fn trim(&mut self, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<(), RusimgError> {
        let mut w = trim_wh.0;
        let mut h = trim_wh.1;
        if self.size.width < (trim_xy.0 + w) as usize || self.size.height < (trim_xy.1 + h) as usize {
            if self.size.width > trim_xy.0 as usize && self.size.height > trim_xy.1 as usize {
                w = if self.size.width < (trim_xy.0 + w) as usize { self.size.width as u32 - trim_xy.0 } else { trim_wh.0 };
                h = if self.size.height < (trim_xy.1 + h) as usize { self.size.height as u32 - trim_xy.1 } else { trim_wh.1 };
                println!("Required width or height is larger than image size. Corrected size: {}x{} -> {}x{}", trim_wh.0, trim_wh.1, w, h);
            }
            else {
                return Err(RusimgError::InvalidTrimXY);
            }
        }

        self.image = self.image.crop(trim_xy.0, trim_xy.1, w, h);

        println!("Trim: {}x{} -> {}x{}", self.size.width, self.size.height, w, h);

        self.size.width = w as usize;
        self.size.height = h as usize;

        Ok(())
    }

    fn grayscale(&mut self) {
        self.image = self.image.grayscale();
        println!("Grayscale: Done.");
    }

    fn view(&self) -> Result<(), RusimgError> {
        let conf_width = self.size.width as f64 / std::cmp::max(self.size.width, self.size.height) as f64 * 100 as f64;
        let conf_height = self.size.height as f64 / std::cmp::max(self.size.width, self.size.height) as f64 as f64 * 50 as f64;
        let conf = viuer::Config {
            absolute_offset: false,
            width: Some(conf_width as u32),
            height: Some(conf_height as u32),    
            ..Default::default()
        };

        viuer::print(&self.image, &conf).map_err(|e| RusimgError::FailedToViewImage(e.to_string()))?;

        Ok(())
    }
}
