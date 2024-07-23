use image::DynamicImage;

use std::fs::Metadata;
use std::path::PathBuf;

use super::{ImgSize, RusimgError, RusimgTrait, Rect};

#[derive(Debug, Clone)]
pub struct BmpImage {
    pub image: DynamicImage,
    size: ImgSize,
    pub metadata_input: Metadata,
    pub metadata_output: Option<Metadata>,
    pub filepath_input: PathBuf,
    pub filepath_output: Option<PathBuf>,
}

impl RusimgTrait for BmpImage {
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

    fn save(&mut self, path: Option<PathBuf>) -> Result<(), RusimgError> {
        let save_path = Self::save_filepath(&self, &self.filepath_input, path, &"bmp".to_string())?;
        self.image.to_rgb8().save(&save_path).map_err(|e| RusimgError::FailedToSaveImage(e.to_string()))?;
        self.metadata_output = Some(std::fs::metadata(&save_path).map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?);
        self.filepath_output = Some(save_path);

        Ok(())
    }

    fn compress(&mut self, _quality: Option<f32>) -> Result<(), RusimgError> {
        Err(RusimgError::ImageFormatCannotBeCompressed)
    }

    fn resize(&mut self, resize_ratio: u8) -> Result<ImgSize, RusimgError> {
        let nwidth = (self.size.width as f32 * (resize_ratio as f32 / 100.0)) as usize;
        let nheight = (self.size.height as f32 * (resize_ratio as f32 / 100.0)) as usize;
        
        self.image = self.image.resize(nwidth as u32, nheight as u32, image::imageops::FilterType::Lanczos3);

        self.size.width = nwidth;
        self.size.height = nheight;

        Ok(self.size)
    }

    fn trim(&mut self, trim: Rect) -> Result<ImgSize, RusimgError> {
        let mut w = trim.w;
        let mut h = trim.h;
        if self.size.width < (trim.x + trim.w) as usize || self.size.height < (trim.y + trim.h) as usize {
            if self.size.width > trim.x as usize && self.size.height > trim.y as usize {
                w = if self.size.width < (trim.x + trim.w) as usize { self.size.width as u32 - trim.x } else { trim.w };
                h = if self.size.height < (trim.y + trim.h) as usize { self.size.height as u32 - trim.y } else { trim.h };
                //println!("Required width or height is larger than image size. Corrected size: {}x{} -> {}x{}", trim_wh.0, trim_wh.1, w, h);
            }
            else {
                return Err(RusimgError::InvalidTrimXY);
            }
        }

        self.image = self.image.crop(trim.x, trim.y, w, h);

        self.size.width = w as usize;
        self.size.height = h as usize;

        Ok(self.size)
    }

    fn grayscale(&mut self) {
        self.image = self.image.grayscale();
    }

    fn set_dynamic_image(&mut self, image: DynamicImage) -> Result<(), RusimgError> {
        self.image = image;
        Ok(())
    }
    
    fn get_dynamic_image(&mut self) -> Result<DynamicImage, RusimgError> {
        Ok(self.image.clone())
    }

    fn get_source_filepath(&self) -> PathBuf {
        self.filepath_input.clone()
    }

    fn get_destination_filepath(&self) -> Option<PathBuf> {
        self.filepath_output.clone()
    }

    fn get_metadata_src(&self) -> Metadata {
        self.metadata_input.clone()
    }

    fn get_metadata_dest(&self) -> Option<Metadata> {
        self.metadata_output.clone()
    }

    fn get_size(&self) -> ImgSize {
        self.size
    }
}
