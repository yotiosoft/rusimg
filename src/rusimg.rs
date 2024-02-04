mod imgprocessor;

use std::path::{Path, PathBuf};
use std::fmt;
use image::DynamicImage;
use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum RusimgError {
    FailedToOpenFile(String),
    FailedToReadFile(String),
    FailedToGetMetadata(String),
    FailedToOpenImage(String),
    FailedToSaveImage(String),
    FailedToCopyBinaryData(String),
    FailedToGetFilename(PathBuf),
    FailedToCreateFile(String),
    FailedToWriteFIle(String),
    FailedToDecodeWebp,
    FailedToEncodeWebp(String),
    FailedToCompressImage(Option<String>),
    FailedToConvertPathToString,
    FailedToViewImage(String),
    InvalidTrimXY,
    BMPImagesCannotBeCompressed,
    UnsupportedFileExtension,
    ImageDataIsNone,
    FailedToGetDynamicImage,
}
impl fmt::Display for RusimgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RusimgError::FailedToOpenFile(s) => write!(f, "Failed to open file: {}", s),
            RusimgError::FailedToReadFile(s) => write!(f, "Failed to read file: {}", s),
            RusimgError::FailedToGetMetadata(s) => write!(f, "Failed to get metadata: {}", s),
            RusimgError::FailedToOpenImage(s) => write!(f, "Failed to open image: {}", s),
            RusimgError::FailedToSaveImage(s) => write!(f, "Failed to save image: {}", s),
            RusimgError::FailedToCopyBinaryData(s) => write!(f, "Failed to copy binary data to memory: {}", s),
            RusimgError::FailedToGetFilename(s) => write!(f, "Failed to get filename: {}", s.display()),
            RusimgError::FailedToCreateFile(s) => write!(f, "Failed to create file: {}", s),
            RusimgError::FailedToWriteFIle(s) => write!(f, "Failed to write file: {}", s),
            RusimgError::FailedToDecodeWebp => write!(f, "Failed to decode webp"),
            RusimgError::FailedToEncodeWebp(s) => write!(f, "Failed to encode webp: {}", s),
            RusimgError::FailedToCompressImage(s) => {
                if let Some(s) = s {
                    write!(f, "Failed to compress image: {}", s)
                }
                else {
                    write!(f, "Failed to compress image")
                }
            }
            RusimgError::FailedToConvertPathToString => write!(f, "Failed to convert path to string"),
            RusimgError::FailedToViewImage(s) => write!(f, "Failed to view image: {}", s),
            RusimgError::InvalidTrimXY => write!(f, "Invalid trim XY"),
            RusimgError::BMPImagesCannotBeCompressed => write!(f, "BMP images cannot be compressed"),
            RusimgError::UnsupportedFileExtension => write!(f, "Unsupported file extension"),
            RusimgError::ImageDataIsNone => write!(f, "Image data is None"),
            RusimgError::FailedToGetDynamicImage => write!(f, "Failed to get dynamic image"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RusimgStatus {
    Success,
    Cancel,
    NotNeeded,
}

#[derive(Debug, Clone, Default)]
pub struct ImgData {
    pub bmp: Option<imgprocessor::bmp::BmpImage>,
    pub jpeg: Option<imgprocessor::jpeg::JpegImage>,
    pub png: Option<imgprocessor::png::PngImage>,
    pub webp: Option<imgprocessor::webp::WebpImage>,
}

#[derive(Debug, Clone)]
pub struct RusImg {
    pub extension: Extension,
    pub data: ImgData,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ImgSize {
    pub width: usize,
    pub height: usize,
}
impl ImgSize {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SaveStatus {
    pub status: RusimgStatus,
    pub output_path: Option<PathBuf>,
    pub before_filesize: u64,
    pub after_filesize: Option<u64>,
}

/// Open an image file and return a RusImg object.
pub fn open_image(path: &Path) -> Result<RusImg, RusimgError> {
    imgprocessor::do_open_image(path)
}

impl RusImg {
    /// Get image size.
    pub fn get_image_size(&self) -> Result<ImgSize, RusimgError> {
        imgprocessor::do_get_image_size(self)
    }

    /// Resize an image.
    /// It must be called after open_image().
    /// Set ratio to 100 to keep the original size.
    pub fn resize(&mut self, ratio: u8) -> Result<ImgSize, RusimgError> {
        let size = imgprocessor::do_resize(self, ratio)?;
        Ok(size)
    }

    /// Trim an image.
    /// It must be called after open_image().
    pub fn trim(&mut self, trim_x: u32, trim_y: u32, trim_w: u32, trim_h: u32) -> Result<ImgSize, RusimgError> {
        let size = imgprocessor::do_trim(self, (trim_x, trim_y), (trim_w, trim_h))?;
        Ok(size)
    }

    /// Grayscale an image.
    /// It must be called after open_image().
    pub fn grayscale(&mut self) -> Result<(), RusimgError> {
        imgprocessor::do_grayscale(self)?;
        Ok(())
    }

    /// Compress an image.
    /// It must be called after open_image().
    /// Set quality to 100 to keep the original quality.
    pub fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError> {
        imgprocessor::do_compress(&mut self.data, &self.extension, quality)?;
        Ok(())
    }

    /// Convert an image to another format.
    /// And replace the original image with the new one.
    /// It must be called after open_image().
    pub fn convert(&mut self, new_extension: Extension) -> Result<(), RusimgError> {
        let new_rusimg = imgprocessor::do_convert(self, &new_extension);
        match new_rusimg {
            Ok(new_rusimg) => {
                self.extension = new_extension;
                self.data = new_rusimg.data;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    /// View an image on the terminal.
    /// It must be called after open_image().
    pub fn view(&mut self) -> Result<(), RusimgError> {
        imgprocessor::do_view(self)?;
        Ok(())
    }

    /// Get a DynamicImage from an Img.
    pub fn get_dynamic_image(&mut self) -> Result<DynamicImage, RusimgError> {
        let dynamic_image = match self.extension {
            Extension::Png => {
                if self.data.png.is_none() {
                    return Err(RusimgError::FailedToGetDynamicImage);
                }
                self.data.png.as_ref().unwrap().image.clone()
            }
            Extension::Jpeg => {
                if self.data.jpeg.is_none() {
                    return Err(RusimgError::FailedToGetDynamicImage);
                }
                self.data.jpeg.as_ref().unwrap().image.clone()
            }
            Extension::Bmp => {
                if self.data.bmp.is_none() {
                    return Err(RusimgError::FailedToGetDynamicImage);
                }
                self.data.bmp.as_ref().unwrap().image.clone()
            }
            Extension::Webp => {
                if self.data.webp.is_none() {
                    return Err(RusimgError::FailedToGetDynamicImage);
                }
                self.data.webp.as_ref().unwrap().image.clone()
            }
        };
        Ok(dynamic_image)
    }

    /// Get file extension.
    pub fn get_extension(&self) -> Extension {
        self.extension.clone()
    }

    /// Get input file path.
    pub fn get_input_filepath(&self) -> PathBuf {
        match self.extension {
            Extension::Png => self.data.png.as_ref().unwrap().filepath_input.clone(),
            Extension::Jpeg => self.data.jpeg.as_ref().unwrap().filepath_input.clone(),
            Extension::Bmp => self.data.bmp.as_ref().unwrap().filepath_input.clone(),
            Extension::Webp => self.data.webp.as_ref().unwrap().filepath_input.clone(),
        }
    }

    /// Save an image to a file.
    /// If path is None, the original file will be overwritten.
    pub fn save_image(&mut self, path: Option<&str>) -> Result<SaveStatus, RusimgError> {
        let path_buf = if let Some(path) = path {
            Some(PathBuf::from(path))
        } else {
            None
        };
        let ret = imgprocessor::do_save_image(path_buf, &mut self.data, &self.extension)?;
        Ok(ret)
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Extension {
    Bmp,
    Jpeg,
    Png,
    Webp,
}
impl fmt::Display for Extension {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Extension::Bmp => write!(f, "bmp"),
            Extension::Jpeg => write!(f, "jpeg"),
            Extension::Png => write!(f, "png"),
            Extension::Webp => write!(f, "webp"),
        }
    }
}
impl Extension {
    pub fn from_str(s: &str) -> Result<Self, RusimgError> {
        match s.to_ascii_lowercase().as_str() {
            "bmp" => Ok(Extension::Bmp),
            "jpeg" | "jpg" => Ok(Extension::Jpeg),
            "png" => Ok(Extension::Png),
            "webp" => Ok(Extension::Webp),
            _ => Err(RusimgError::UnsupportedFileExtension),
        }
    }

    pub fn to_image_format(&self) -> image::ImageFormat {
        match self {
            Extension::Bmp => image::ImageFormat::Bmp,
            Extension::Jpeg => image::ImageFormat::Jpeg,
            Extension::Png => image::ImageFormat::Png,
            Extension::Webp => image::ImageFormat::WebP,
        }
    }
}
