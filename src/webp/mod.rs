use image::{DynamicImage, EncodableLayout};

use std::fs::Metadata;
use std::io::Write;
use std::path::{PathBuf, Path};

use super::{RusimgTrait, RusimgError, ImgSize, Rect};

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
    pub filepath_input: PathBuf,
    pub filepath_output: Option<PathBuf>,
}

impl RusimgTrait for WebpImage {
    /// Import an image from a DynamicImage object.
    fn import(image: DynamicImage, source_path: PathBuf, source_metadata: Metadata) -> Result<Self, RusimgError> {
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

    /// Open an image from a image buffer.
    fn open(path: PathBuf, image_buf: Vec<u8>, metadata: Metadata) -> Result<Self, RusimgError> {
        let webp_decoder = dep_webp::Decoder::new(&image_buf).decode();
        if let Some(webp_decoder) = webp_decoder {
            let image = webp_decoder.to_image();
            let (width, height) = (image.width() as usize, image.height() as usize);

            Ok(Self {
                image,
                image_bytes: Some(image_buf),
                width,
                height,
                operations_count: 0,
                required_quality: None,
                metadata_input: metadata,
                metadata_output: None,
                filepath_input: path,
                filepath_output: None,
            })
        }
        else {
            return Err(RusimgError::FailedToDecodeWebp);
        }
    }

    /// Save the image to a file.
    fn save(&mut self, path: Option<PathBuf>) -> Result<(), RusimgError> {
        let save_path = Self::get_save_filepath(&self, &self.filepath_input, path, &"webp".to_string())?;

        // 元が webp かつ操作回数が 0 なら encode しない
        let source_is_webp = Path::new(&self.filepath_input).extension().and_then(|s| s.to_str()).unwrap_or("").to_string() == "webp";
        if source_is_webp && self.operations_count == 0 && self.image_bytes.is_some() {
            let mut file = std::fs::File::create(&save_path).map_err(|e| RusimgError::FailedToCreateFile(e.to_string()))?;
            file.write_all(self.image_bytes.as_ref().unwrap()).map_err(|e| RusimgError::FailedToWriteFIle(e.to_string()))?;

            self.metadata_output = Some(file.metadata().map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?);
            self.filepath_output = Some(save_path);

            return Ok(());
        }

        // quality
        let quality = if let Some(q) = self.required_quality {
            q       // 指定されていればその値
        }
        else {
            75.0    // 既定: 100.0（最高品質, compress を必要としない場合）
        };
       
        // DynamicImage を （圧縮＆）保存
        let encoded_webp = dep_webp::Encoder::from_rgba(&self.image.to_rgba8(), self.image.width(), self.image.height()).encode(quality);

        let mut file = std::fs::File::create(&save_path).map_err(|e| RusimgError::FailedToCreateFile(e.to_string()))?;
        file.write_all(&encoded_webp.as_bytes()).map_err(|e| RusimgError::FailedToWriteFIle(e.to_string()))?;

        self.metadata_output = Some(file.metadata().map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?);
        self.filepath_output = Some(save_path);

        Ok(())
    }

    /// Compress the image.
    /// quality: Option<f32> 0.0 - 100.0
    /// Because the webp crate compresses the image when saving it, the compress() method does not need to do anything.
    /// So this method only sets the quality value.
    fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError> {
        // compress later when saving
        self.required_quality = quality;
        self.operations_count += 1;
        Ok(())
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
