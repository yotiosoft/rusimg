mod bmp;
mod jpeg;
mod png;
mod webp;

use std::fs::Metadata;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::fmt;
use image::DynamicImage;

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
    ImageFormatCannotBeCompressed,
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
            RusimgError::ImageFormatCannotBeCompressed => write!(f, "this image format cannot be compressed"),
            RusimgError::UnsupportedFileExtension => write!(f, "Unsupported file extension"),
            RusimgError::ImageDataIsNone => write!(f, "Image data is None"),
            RusimgError::FailedToGetDynamicImage => write!(f, "Failed to get dynamic image"),
        }
    }
}

pub struct RusImg {
    pub extension: Extension,
    pub data: Box<(dyn RusimgTrait)>,
}

pub trait RusimgTrait {
    fn import(image: DynamicImage, source_path: PathBuf, source_metadata: Metadata) -> Result<Self, RusimgError> where Self: Sized;
    fn open(path: PathBuf, image_buf: Vec<u8>, metadata: Metadata) -> Result<Self, RusimgError> where Self: Sized;
    fn save(&mut self, path: Option<PathBuf>) -> Result<(), RusimgError>;
    fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError>;
    fn resize(&mut self, resize_ratio: u8) -> Result<ImgSize, RusimgError>;
    fn trim(&mut self, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<ImgSize, RusimgError>;
    fn grayscale(&mut self);
    fn view(&self) -> Result<(), RusimgError>;

    fn set_dynamic_image(&mut self, image: DynamicImage) -> Result<(), RusimgError>;

    fn get_dynamic_image(&mut self) -> Result<DynamicImage, RusimgError>;
    fn get_source_filepath(&self) -> PathBuf;
    fn get_destination_filepath(&self) -> Option<PathBuf>;
    fn get_metadata_src(&self) -> Metadata;
    fn get_metadata_dest(&self) -> Option<Metadata>;
    fn get_size(&self) -> ImgSize;

    fn save_filepath(&self, source_filepath: &PathBuf, destination_filepath: Option<PathBuf>, new_extension: &String) -> Result<PathBuf, RusimgError> {
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
}

#[derive(Debug, Clone, PartialEq, Copy, Default)]
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
    pub output_path: Option<PathBuf>,
    pub before_filesize: u64,
    pub after_filesize: Option<u64>,
}

// 画像フォーマットを取得
fn guess_image_format(image_buf: &[u8]) -> Result<image::ImageFormat, RusimgError> {
    let format = image::guess_format(image_buf).map_err(|e| RusimgError::FailedToOpenImage(e.to_string()))?;
    Ok(format)
}

/// Open an image file and return a RusImg object.
pub fn open_image(path: &Path) -> Result<RusImg, RusimgError> {
    let mut raw_data = std::fs::File::open(&path.to_path_buf()).map_err(|e| RusimgError::FailedToOpenFile(e.to_string()))?;
    let mut buf = Vec::new();
    raw_data.read_to_end(&mut buf).map_err(|e| RusimgError::FailedToReadFile(e.to_string()))?;
    let metadata_input = raw_data.metadata().map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?;

    match guess_image_format(&buf)? {
        image::ImageFormat::Bmp => {
            let image = bmp::BmpImage::open(path.to_path_buf(), buf, metadata_input)?;
            let data = Box::new(image);
            Ok(RusImg { extension: Extension::Bmp, data: data })
        },
        image::ImageFormat::Jpeg => {
            let image = jpeg::JpegImage::open(path.to_path_buf(), buf, metadata_input)?;
            let data = Box::new(image);
            Ok(RusImg { extension: Extension::Jpeg, data: data })
        },
        image::ImageFormat::Png => {
            let image = png::PngImage::open(path.to_path_buf(), buf, metadata_input)?;
            let data = Box::new(image);
            Ok(RusImg { extension: Extension::Png, data: data })
        },
        image::ImageFormat::WebP => {
            let image = webp::WebpImage::open(path.to_path_buf(), buf, metadata_input)?;
            let data = Box::new(image);
            Ok(RusImg { extension: Extension::Webp, data: data })
        },
        _ => Err(RusimgError::UnsupportedFileExtension),
    }
}

impl RusImg {
    /// Get image size.
    pub fn get_image_size(&self) -> Result<ImgSize, RusimgError> {
        let size = self.data.get_size();
        Ok(size)
    }

    /// Resize an image.
    /// It must be called after open_image().
    /// Set ratio to 100 to keep the original size.
    pub fn resize(&mut self, ratio: u8) -> Result<ImgSize, RusimgError> {
        let size = self.data.resize(ratio)?;
        Ok(size)
    }

    /// Trim an image.
    /// It must be called after open_image().
    pub fn trim(&mut self, trim_x: u32, trim_y: u32, trim_w: u32, trim_h: u32) -> Result<ImgSize, RusimgError> {
        let size = self.data.trim((trim_x, trim_y), (trim_w, trim_h))?;
        Ok(size)
    }

    /// Grayscale an image.
    /// It must be called after open_image().
    pub fn grayscale(&mut self) -> Result<(), RusimgError> {
        self.data.grayscale();
        Ok(())
    }

    /// Compress an image.
    /// It must be called after open_image().
    /// Set quality to 100 to keep the original quality.
    pub fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError> {
        self.data.compress(quality)?;
        Ok(())
    }

    /// Convert an image to another format.
    /// And replace the original image with the new one.
    /// It must be called after open_image().
    pub fn convert(&mut self, new_extension: Extension) -> Result<(), RusimgError> {
        let dynamic_image = self.data.get_dynamic_image()?;
        let filepath = self.data.get_source_filepath();
        let metadata = self.data.get_metadata_src();

        let new_image: Box<(dyn RusimgTrait)> = match new_extension {
            Extension::Bmp => {
                let bmp = bmp::BmpImage::import(dynamic_image, filepath, metadata)?;
                Box::new(bmp)
            },
            Extension::Jpeg => {
                let jpeg = jpeg::JpegImage::import(dynamic_image, filepath, metadata)?;
                Box::new(jpeg)
            },
            Extension::Png => {
                let png = png::PngImage::import(dynamic_image, filepath, metadata)?;
                Box::new(png)
            },
            Extension::Webp => {
                let webp = webp::WebpImage::import(dynamic_image, filepath, metadata)?;
                Box::new(webp)
            },
            Extension::ExternalFormat(s) => return Err(RusimgError::UnsupportedFileExtension),
        };

        self.extension = new_extension;
        self.data = new_image;

        Ok(())
    }

    /// View an image on the terminal.
    /// It must be called after open_image().
    pub fn view(&mut self) -> Result<(), RusimgError> {
        self.data.view()
    }

    /// Set a DynamicImage to an Img.
    pub fn set_dynamic_image(&mut self, image: DynamicImage) -> Result<(), RusimgError> {
        self.data.set_dynamic_image(image)?;
        Ok(())
    }

    /// Get a DynamicImage from an Img.
    pub fn get_dynamic_image(&mut self) -> Result<DynamicImage, RusimgError> {
        let dynamic_image = self.data.get_dynamic_image()?;
        Ok(dynamic_image)
    }

    /// Get file extension.
    pub fn get_extension(&self) -> Extension {
        self.extension.clone()
    }

    /// Get input file path.
    pub fn get_input_filepath(&self) -> PathBuf {
        self.data.get_source_filepath()
    }

    /// Save an image to a file.
    /// If path is None, the original file will be overwritten.
    pub fn save_image(&mut self, path: Option<&str>) -> Result<SaveStatus, RusimgError> {
        let path_buf = match path {
            Some(p) => Some(PathBuf::from(p)),
            None => None,
        };
        self.data.save(path_buf)?;

        let ret = SaveStatus {
            output_path: self.data.get_destination_filepath().clone().or(None),
            before_filesize: self.data.get_metadata_src().len(),
            after_filesize: self.data.get_metadata_dest().as_ref().or(None).map(|m| m.len())
        };
        Ok(ret)
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Extension {
    Bmp,
    Jpeg,
    Png,
    Webp,
    ExternalFormat(String),
}
impl fmt::Display for Extension {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Extension::Bmp => write!(f, "bmp"),
            Extension::Jpeg => write!(f, "jpeg"),
            Extension::Png => write!(f, "png"),
            Extension::Webp => write!(f, "webp"),
            Extension::ExternalFormat(s) => write!(f, "{}", s),
        }
    }
}
