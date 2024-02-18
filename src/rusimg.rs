#[cfg(feature="bmp")]
mod bmp;
#[cfg(feature="jpeg")]
mod jpeg;
#[cfg(feature="png")]
mod png;
#[cfg(feature="webp")]
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

/// Open specified image file format and return a RusImg object.
#[cfg(feature="bmp")]
pub fn open_bmp_image(path: &Path, buf: Vec<u8>, metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    let image = bmp::BmpImage::open(path.to_path_buf(), buf, metadata_input)?;
    let data = Box::new(image);
    Ok(RusImg { extension: Extension::Bmp, data: data })
}
#[cfg(not(feature="bmp"))]
pub fn open_bmp_image(_path: &Path, _buf: Vec<u8>, _metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}
#[cfg(feature="jpeg")]
pub fn open_jpeg_image(path: &Path, buf: Vec<u8>, metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    let image = jpeg::JpegImage::open(path.to_path_buf(), buf, metadata_input)?;
    let data = Box::new(image);
    Ok(RusImg { extension: Extension::Jpeg, data: data })
}
#[cfg(not(feature="jpeg"))]
pub fn open_jpeg_image(_path: &Path, _buf: Vec<u8>, _metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}
#[cfg(feature="png")]
pub fn open_png_image(path: &Path, buf: Vec<u8>, metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    let image = png::PngImage::open(path.to_path_buf(), buf, metadata_input)?;
    let data = Box::new(image);
    Ok(RusImg { extension: Extension::Png, data: data })
}
#[cfg(not(feature="png"))]
pub fn open_png_image(_path: &Path, _buf: Vec<u8>, _metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}
#[cfg(feature="webp")]
pub fn open_webp_image(path: &Path, buf: Vec<u8>, metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    let image = webp::WebpImage::open(path.to_path_buf(), buf, metadata_input)?;
    let data = Box::new(image);
    Ok(RusImg { extension: Extension::Webp, data: data })
}
#[cfg(not(feature="webp"))]
pub fn open_webp_image(_path: &Path, _buf: Vec<u8>, _metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}

/// Open an image file and return a RusImg object.
pub fn open_image(path: &Path) -> Result<RusImg, RusimgError> {
    let mut raw_data = std::fs::File::open(&path.to_path_buf()).map_err(|e| RusimgError::FailedToOpenFile(e.to_string()))?;
    let mut buf = Vec::new();
    raw_data.read_to_end(&mut buf).map_err(|e| RusimgError::FailedToReadFile(e.to_string()))?;
    let metadata_input = raw_data.metadata().map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?;

    match guess_image_format(&buf)? {
        image::ImageFormat::Bmp => {
            open_bmp_image(path, buf, metadata_input)
        },
        image::ImageFormat::Jpeg => {
            open_jpeg_image(path, buf, metadata_input)
        },
        image::ImageFormat::Png => {
            open_png_image(path, buf, metadata_input)
        },
        image::ImageFormat::WebP => {
            open_webp_image(path, buf, metadata_input)
        },
        _ => Err(RusimgError::UnsupportedFileExtension),
    }
}

/// Converter interfaces
#[cfg(feature="bmp")]
pub fn convert_to_bmp_image(dynamic_image: DynamicImage, filepath: PathBuf, metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    let bmp = bmp::BmpImage::import(dynamic_image, filepath, metadata)?;
    Ok(Box::new(bmp))
}
#[cfg(not(feature="bmp"))]
pub fn convert_to_bmp_image(_dynamic_image: DynamicImage, _filepath: PathBuf, _metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}
#[cfg(feature="jpeg")]
pub fn convert_to_jpeg_image(dynamic_image: DynamicImage, filepath: PathBuf, metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    let jpeg = jpeg::JpegImage::import(dynamic_image, filepath, metadata)?;
    Ok(Box::new(jpeg))
}
#[cfg(not(feature="jpeg"))]
pub fn convert_to_jpeg_image(_dynamic_image: DynamicImage, _filepath: PathBuf, _metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}
#[cfg(feature="png")]
pub fn convert_to_png_image(dynamic_image: DynamicImage, filepath: PathBuf, metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    let png = png::PngImage::import(dynamic_image, filepath, metadata)?;
    Ok(Box::new(png))
}
#[cfg(not(feature="png"))]
pub fn convert_to_png_image(_dynamic_image: DynamicImage, _filepath: PathBuf, _metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}
#[cfg(feature="webp")]
pub fn convert_to_webp_image(dynamic_image: DynamicImage, filepath: PathBuf, metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    let webp = webp::WebpImage::import(dynamic_image, filepath, metadata)?;
    Ok(Box::new(webp))
}
#[cfg(not(feature="webp"))]
pub fn convert_to_webp_image(_dynamic_image: DynamicImage, _filepath: PathBuf, _metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
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
                convert_to_bmp_image(dynamic_image, filepath, metadata)?
            },
            Extension::Jpeg => {
                convert_to_jpeg_image(dynamic_image, filepath, metadata)?
            },
            Extension::Png => {
                convert_to_png_image(dynamic_image, filepath, metadata)?
            },
            Extension::Webp => {
                convert_to_webp_image(dynamic_image, filepath, metadata)?
            },
            Extension::ExternalFormat(_) => return Err(RusimgError::UnsupportedFileExtension),
        };

        self.extension = new_extension;
        self.data = new_image;

        Ok(())
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
