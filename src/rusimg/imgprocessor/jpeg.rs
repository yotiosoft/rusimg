use mozjpeg::{Compress, ColorSpace, ScanMode};
use image::DynamicImage;

use std::fs::Metadata;
use std::io::Write;
use std::path::PathBuf;

use super::{RusimgTrait, RusimgError, ImgSize};

#[derive(Debug, Clone)]
pub struct JpegImage {
    pub image: DynamicImage,
    image_bytes: Option<Vec<u8>>,
    size: ImgSize,
    operations_count: u32,
    extension_str: String,
    pub metadata_input: Metadata,
    pub metadata_output: Option<Metadata>,
    pub filepath_input: PathBuf,
    pub filepath_output: Option<PathBuf>,
}

impl RusimgTrait for JpegImage {
    fn import(image: DynamicImage, source_path: PathBuf, source_metadata: Metadata) -> Result<Self, RusimgError> {
        let size = ImgSize { width: image.width() as usize, height: image.height() as usize };

        Ok(Self {
            image,
            image_bytes: None,
            size,
            operations_count: 0,
            extension_str: "jpg".to_string(),
            metadata_input: source_metadata,
            metadata_output: None,
            filepath_input: source_path,
            filepath_output: None,
        })
    }

    fn open(path: PathBuf, image_buf: Vec<u8>, metadata: Metadata) -> Result<Self, RusimgError> {
        let image = image::load_from_memory(&image_buf).map_err(|e| RusimgError::FailedToOpenImage(e.to_string()))?;
        let size = ImgSize { width: image.width() as usize, height: image.height() as usize };

        let extension_str = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_string();

        Ok(Self {
            image,
            image_bytes: None,
            size,
            operations_count: 0,
            extension_str,
            metadata_input: metadata,
            metadata_output: None,
            filepath_input: path,
            filepath_output: None,
        })
    }

    fn save(&mut self, path: Option<PathBuf>) -> Result<(), RusimgError> {
        let save_path = Self::save_filepath(&self, &self.filepath_input, path, &self.extension_str)?;
        
        // image_bytes == None の場合、DynamicImage を 保存
        if self.image_bytes.is_none() {
            self.image.to_rgba8().save(&save_path).map_err(|e| RusimgError::FailedToSaveImage(e.to_string()))?;
            self.metadata_output = Some(std::fs::metadata(&save_path).map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?);
        }
        // image_bytes != None の場合、mozjpeg::Compress で圧縮したバイナリデータを保存
        else {
            let mut file = std::fs::File::create(&save_path).map_err(|e| RusimgError::FailedToCreateFile(e.to_string()))?;
            file.write_all(&self.image_bytes.as_ref().unwrap()).map_err(|e| RusimgError::FailedToWriteFIle(e.to_string()))?;
            self.metadata_output = Some(file.metadata().map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?);
        }

        self.filepath_output = Some(save_path);

        Ok(())
    }

    fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError> {
        let quality = quality.unwrap_or(75.0);  // default quality: 75.0

        let image_bytes = self.image.clone().into_bytes();

        let mut compress = Compress::new(ColorSpace::JCS_RGB);
        compress.set_scan_optimization_mode(ScanMode::AllComponentsTogether);
        compress.set_size(self.size.width, self.size.height);
        compress.set_mem_dest();
        compress.set_quality(quality);
        compress.start_compress();
        compress.write_scanlines(&image_bytes);
        compress.finish_compress();

        self.image_bytes = Some(compress.data_to_vec().map_err(|_| RusimgError::FailedToCompressImage(None))?);

        self.operations_count += 1;

        Ok(())
    }

    fn resize(&mut self, resize_ratio: u8) -> Result<ImgSize, RusimgError> {
        let nwidth = (self.size.width as f32 * (resize_ratio as f32 / 100.0)) as usize;
        let nheight = (self.size.height as f32 * (resize_ratio as f32 / 100.0)) as usize;
        
        self.image = self.image.resize(nwidth as u32, nheight as u32, image::imageops::FilterType::Lanczos3);

        self.size.width = nwidth;
        self.size.height = nheight;

        self.operations_count += 1;
        Ok(self.size)
    }

    fn trim(&mut self, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<ImgSize, RusimgError> {
        let mut w = trim_wh.0;
        let mut h = trim_wh.1;
        if self.size.width < (trim_xy.0 + w) as usize || self.size.height < (trim_xy.1 + h) as usize {
            if self.size.width > trim_xy.0 as usize && self.size.height > trim_xy.1 as usize {
                w = if self.size.width < (trim_xy.0 + w) as usize { self.size.width as u32 - trim_xy.0 } else { trim_wh.0 };
                h = if self.size.height < (trim_xy.1 + h) as usize { self.size.height as u32 - trim_xy.1 } else { trim_wh.1 };
                //println!("Required width or height is larger than image size. Corrected size: {}x{} -> {}x{}", trim_wh.0, trim_wh.1, w, h);
            }
            else {
                return Err(RusimgError::InvalidTrimXY);
            }
        }

        self.image = self.image.crop(trim_xy.0, trim_xy.1, w, h);

        self.size.width = w as usize;
        self.size.height = h as usize;

        self.operations_count += 1;
        Ok(self.size)
    }

    fn grayscale(&mut self) {
        self.image = self.image.grayscale();
        self.operations_count += 1;
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
