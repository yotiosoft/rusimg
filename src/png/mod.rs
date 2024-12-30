use std::io::{Write, Cursor};
use std::fs::Metadata;
use std::path::PathBuf;
use image::DynamicImage;

use super::{RusimgTrait, RusimgError, ImgSize, Rect};

#[derive(Debug, Clone)]
pub struct PngImage {
    binary_data: Vec<u8>,
    pub image: DynamicImage,
    image_bytes: Option<Vec<u8>>,
    width: usize,
    height: usize,
    operations_count: u32,
    pub metadata_input: Metadata,
    pub metadata_output: Option<Metadata>,
    pub filepath_input: PathBuf,
    pub filepath_output: Option<PathBuf>,
}

impl RusimgTrait for PngImage {
    /// Import an image from a DynamicImage object.
    fn import(image: DynamicImage, source_path: PathBuf, source_metadata: Metadata) -> Result<Self, RusimgError> {
        let (width, height) = (image.width() as usize, image.height() as usize);

        let mut new_binary_data = Vec::new();
        image.write_to(&mut Cursor::new(&mut new_binary_data), image::ImageFormat::Png)
            .map_err(|e| RusimgError::FailedToCopyBinaryData(e.to_string()))?;

        Ok(Self {
            binary_data: new_binary_data,
            image,
            image_bytes: None,
            width,
            height,
            operations_count: 0,
            metadata_input: source_metadata,
            metadata_output: None,
            filepath_input: source_path,
            filepath_output: None,
        })
    }

    /// Open an image from a image buffer.
    fn open(path: PathBuf, image_buf: Vec<u8>, metadata: Metadata) -> Result<Self, RusimgError> {
        let image = image::load_from_memory(&image_buf).map_err(|e| RusimgError::FailedToOpenImage(e.to_string()))?;
        let (width, height) = (image.width() as usize, image.height() as usize);

        Ok(Self {
            binary_data: image_buf,
            image,
            image_bytes: None,
            width,
            height,
            operations_count: 0,
            metadata_input: metadata,
            metadata_output: None,
            filepath_input: path,
            filepath_output: None,
        })
    }

    /// Save the image to a file.
    fn save(&mut self, path: Option<PathBuf>) -> Result<(), RusimgError> {
        let save_path = Self::get_save_filepath(&self, &self.filepath_input, path, &"png".to_string())?;
        
        // image_bytes == None の場合、DynamicImage を 保存
        if self.image_bytes.is_none() {
            self.image.to_rgba8().save(&save_path).map_err(|e| RusimgError::FailedToSaveImage(e.to_string()))?;
            self.metadata_output = Some(std::fs::metadata(&save_path).map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?);
        }
        // image_bytes != None の場合、oxipng で圧縮したバイナリデータを保存
        else {
            let mut file = std::fs::File::create(&save_path).map_err(|e| RusimgError::FailedToCreateFile(e.to_string()))?;
            file.write_all(&self.image_bytes.as_ref().unwrap()).map_err(|e| RusimgError::FailedToWriteFIle(e.to_string()))?;
            self.metadata_output = Some(file.metadata().map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?);
        }

        self.filepath_output = Some(save_path);

        Ok(())
    }

    /// Compress the image.
    /// quality: Option<f32> 0.0 - 100.0
    /// Because oxipng supports only 6 levels of compression, the quality value is converted to a level value.
    fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError> {
        // quality の値に応じて level を設定
        let level = if let Some(q) = quality {
            if q <= 17.0 {
                1
            }
            else if q > 17.0 && q <= 34.0 {
                2
            }
            else if q > 34.0 && q <= 51.0 {
                3
            }
            else if q > 51.0 && q <= 68.0 {
                4
            }
            else if q > 68.0 && q <= 85.0 {
                5
            }
            else {
                6
            }
        }
        else {
            5       // default
        };

        match oxipng::optimize_from_memory(&self.binary_data, &oxipng::Options::from_preset(level)) {
            Ok(data) => {
                self.image_bytes = Some(data);
                self.operations_count += 1;
                Ok(())
            },
            Err(e) => {
                let oxipng_err = match e {
                    oxipng::PngError::DeflatedDataTooLong(s) => Err(format!("(oxipng) deflated data too long: {}", s)),
                    oxipng::PngError::TimedOut => Err("(oxipng) timed out".to_string()),
                    oxipng::PngError::NotPNG => Err("(oxipng) not png".to_string()),
                    oxipng::PngError::APNGNotSupported => Err("(oxipng) apng not supported".to_string()),
                    oxipng::PngError::InvalidData => Err("(oxipng) invalid data".to_string()),
                    oxipng::PngError::TruncatedData => Err("(oxipng) truncated data".to_string()),
                    oxipng::PngError::ChunkMissing(s) => Err(format!("(oxipng) chunk missing: {}", s)),
                    oxipng::PngError::Other(s) => Err(format!("(oxipng) other: {}", s)),
                    _ => Err("unknown error".to_string()),
                };
                Err(RusimgError::FailedToCompressImage(oxipng_err.unwrap()))
            }
        }
    }

    /// Resize the image.
    /// Set the resize_ratio between 1 and 100.
    fn resize(&mut self, resize_ratio: u8) -> Result<ImgSize, RusimgError> {
        let nwidth = (self.width as f32 * (resize_ratio as f32 / 100.0)) as usize;
        let nheight = (self.height as f32 * (resize_ratio as f32 / 100.0)) as usize;

        self.image = self.image.resize(nwidth as u32, nheight as u32, image::imageops::FilterType::Lanczos3);

        self.width = nwidth;
        self.height = nheight;

        self.operations_count += 1;
        Ok(ImgSize::new(self.width, self.height))
    }

    /// Trim the image.
    /// trim: rusimg::Rect { x: u32, y: u32, w: u32, h: u32 }
    fn trim(&mut self, trim: Rect) -> Result<ImgSize, RusimgError> {
        let mut w = trim.w;
        let mut h = trim.h;
        if self.width < (trim.x + trim.w) as usize || self.height < (trim.y + trim.h) as usize {
            if self.width > trim.x as usize && self.height > trim.y as usize {
                w = if self.width < (trim.x + trim.w) as usize { self.width as u32 - trim.x } else { trim.w };
                h = if self.height < (trim.y + trim.h) as usize { self.height as u32 - trim.y } else { trim.h };
                //println!("Required width or height is larger than image size. Corrected size: {}x{} -> {}x{}", trim_wh.0, trim_wh.1, w, h);
            }
            else {
                return Err(RusimgError::InvalidTrimXY);
            }
        }

        self.image = self.image.crop(trim.x, trim.y, w, h);

        self.width = w as usize;
        self.height = h as usize;

        Ok(ImgSize::new(self.width, self.height))
    }

    /// Convert the image to grayscale.
    fn grayscale(&mut self) {
        self.image = self.image.grayscale();
        self.operations_count += 1;
    }

    /// Set the image to a DynamicImage object.
    fn set_dynamic_image(&mut self, image: DynamicImage) -> Result<(), RusimgError> {
        self.image = image;
        Ok(())
    }

    /// Get the DynamicImage object.
    fn get_dynamic_image(&mut self) -> Result<DynamicImage, RusimgError> {
        Ok(self.image.clone())
    }

    /// Get the source file path.
    fn get_source_filepath(&self) -> PathBuf {
        self.filepath_input.clone()
    }

    /// Get the destination file path.
    fn get_destination_filepath(&self) -> Option<PathBuf> {
        self.filepath_output.clone()
    }

    /// Get the source metadata.
    fn get_metadata_src(&self) -> Metadata {
        self.metadata_input.clone()
    }

    /// Get the destination metadata.
    fn get_metadata_dest(&self) -> Option<Metadata> {
        self.metadata_output.clone()
    }

    /// Get the image size.
    fn get_size(&self) -> ImgSize {
        ImgSize::new(self.width, self.height)
    }
}
