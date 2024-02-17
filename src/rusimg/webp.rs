use image::{DynamicImage, EncodableLayout};

use std::fs::Metadata;
use std::io::Write;
use std::path::{PathBuf, Path};

use super::{RusimgTrait, RusimgError, ImgSize};

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

    fn save(&mut self, path: Option<PathBuf>) -> Result<(), RusimgError> {
        let save_path = Self::save_filepath(&self, &self.filepath_input, path, &"webp".to_string())?;

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
            100.0    // 既定: 100.0（最高品質, compress を必要としない場合）
        };
       
        // DynamicImage を （圧縮＆）保存
        let encoded_webp = dep_webp::Encoder::from_rgba(&self.image.to_rgba8(), self.image.width(), self.image.height()).encode(quality);

        let mut file = std::fs::File::create(&save_path).map_err(|e| RusimgError::FailedToCreateFile(e.to_string()))?;
        file.write_all(&encoded_webp.as_bytes()).map_err(|e| RusimgError::FailedToWriteFIle(e.to_string()))?;

        self.metadata_output = Some(file.metadata().map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?);
        self.filepath_output = Some(save_path);

        Ok(())
    }

    fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError> {
        // webp の場合、圧縮は save() で行う
        self.required_quality = quality;
        self.operations_count += 1;
        Ok(())
    }

    fn resize(&mut self, resize_ratio: u8) -> Result<ImgSize, RusimgError> {
        let nwidth = (self.width as f32 * (resize_ratio as f32 / 100.0)) as usize;
        let nheight = (self.height as f32 * (resize_ratio as f32 / 100.0)) as usize;

        self.image = self.image.resize(nwidth as u32, nheight as u32, image::imageops::FilterType::Lanczos3);

        self.width = nwidth;
        self.height = nheight;

        self.operations_count += 1;
        Ok(ImgSize::new(self.width, self.height))
    }

    fn trim(&mut self, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<ImgSize, RusimgError> {
        let mut w = trim_wh.0;
        let mut h = trim_wh.1;
        if self.width < (trim_xy.0 + w) as usize || self.height < (trim_xy.1 + h) as usize {
            if self.width > trim_xy.0 as usize && self.height > trim_xy.1 as usize {
                w = if self.width < (trim_xy.0 + w) as usize { self.width as u32 - trim_xy.0 } else { trim_wh.0 };
                h = if self.height < (trim_xy.1 + h) as usize { self.height as u32 - trim_xy.1 } else { trim_wh.1 };
                //println!("Required width or height is larger than image size. Corrected size: {}x{} -> {}x{}", trim_wh.0, trim_wh.1, w, h);
            }
            else {
                return Err(RusimgError::InvalidTrimXY);
            }
        }

        self.image = self.image.crop(trim_xy.0, trim_xy.1, w, h);

        self.width = w as usize;
        self.height = h as usize;

        self.operations_count += 1;
        Ok(ImgSize::new(self.width, self.height))
    }

    fn grayscale(&mut self) {
        self.image = self.image.grayscale();
        self.operations_count += 1;
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
        ImgSize::new(self.width, self.height)
    }
}
