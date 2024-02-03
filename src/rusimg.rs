mod imgprocessor;

use std::path::{Path, PathBuf};
use std::fs::Metadata;
use std::fmt;
use image::DynamicImage;
use std::io::{stdout, Write};

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
pub enum FileOverwriteAsk {
    YesToAll,
    NoToAll,
    AskEverytime,
}

/// Open an image file and return a RusImg object.
pub fn open_image(path: &Path) -> Result<RusImg, RusimgError> {
    imgprocessor::do_open_image(path)
}

/// Only use by main.rs
pub fn do_save_image(path: Option<PathBuf>, data: &mut ImgData, extension: &Extension, file_overwrite_ask: FileOverwriteAsk) -> Result<(RusimgStatus, Option<PathBuf>, PathBuf, u64, Option<u64>), RusimgError> {
    imgprocessor::do_save_image(path, data, extension, file_overwrite_ask)
}

pub trait RusimgTrait {
    fn import(image: DynamicImage, source_path: PathBuf, source_metadata: Metadata) -> Result<Self, RusimgError> where Self: Sized;
    fn open(path: PathBuf, image_buf: Vec<u8>, metadata: Metadata) -> Result<Self, RusimgError> where Self: Sized;
    fn save(&mut self, path: Option<PathBuf>, file_overwrite_ask: &FileOverwriteAsk) -> Result<RusimgStatus, RusimgError>;
    fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError>;
    fn resize(&mut self, resize_ratio: u8) -> Result<ImgSize, RusimgError>;
    fn trim(&mut self, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<ImgSize, RusimgError>;
    fn grayscale(&mut self);
    fn view(&self) -> Result<(), RusimgError>;

    fn save_filepath(source_filepath: &PathBuf, destination_filepath: Option<PathBuf>, new_extension: &String) -> Result<PathBuf, RusimgError> {
        if let Some(path) = destination_filepath {
            if Path::new(&path).is_dir() {
                let filename = match Path::new(&source_filepath).file_name() {
                    Some(filename) => filename,
                    None => return Err(RusimgError::FailedToGetFilename(source_filepath.clone())),
                };
                Ok(Path::new(&path).join(filename).with_extension(new_extension))
            }
            else {
                Ok(path)
            }
        }
        else {
            Ok(Path::new(&source_filepath).with_extension(new_extension))
        }
    }

    fn check_file_exists(path: &PathBuf, file_overwrite_ask: &FileOverwriteAsk) -> bool {
        // ファイルの存在チェック
        // ファイルが存在する場合、上書きするかどうかを確認
        if Path::new(path).exists() {
            print!("The image file \"{}\" already exists.", path.display());
            match file_overwrite_ask {
                FileOverwriteAsk::YesToAll => {
                    println!(" -> Overwrite it.");
                    return true
                },
                FileOverwriteAsk::NoToAll => {
                    println!(" -> Skip it.");
                    return false
                },
                FileOverwriteAsk::AskEverytime => {},
            }

            print!(" Do you want to overwrite it? [y/N]: ");
            loop {
                stdout().flush().unwrap();

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_ascii_lowercase() == "y" || input.trim().to_ascii_lowercase() == "yes" {
                    return true;
                }
                else if input.trim().to_ascii_lowercase() == "n" || input.trim().to_ascii_lowercase() == "no" || input.trim() == "" {
                    return false;
                }
                else {
                    print!("Please enter y or n: ");
                }
            }
        }
        return true;
    }
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
    /// It must be called after open_image().
    pub fn convert(&mut self, new_extension: Extension) -> Result<(), RusimgError> {
        imgprocessor::do_convert(self, &new_extension)?;
        Ok(())
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

    /// Save an image to a file.
    /// If path is None, the original file will be overwritten.
    pub fn save_image(&mut self, path: Option<&str>) -> Result<(), RusimgError> {
        let path_buf = if let Some(path) = path {
            Some(PathBuf::from(path))
        } else {
            None
        };
        _ = imgprocessor::do_save_image(path_buf, &mut self.data, &self.extension, FileOverwriteAsk::YesToAll)?;
        Ok(())
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
