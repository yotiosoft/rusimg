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

/// Error type for Rusimg.
/// This error type is used in Rusimg functions.
/// Some error types have a string parameter to store the error message.
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
    InvalidTrimXY,
    ImageFormatCannotBeCompressed,
    UnsupportedFileExtension,
    ImageDataIsNone,
    FailedToGetDynamicImage,
    FailedToConvertExtension,
}
/// Implement Display trait for RusimgError.
impl fmt::Display for RusimgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RusimgError::FailedToOpenFile(s) => write!(f, "Failed to open file: \n\t{}", s),
            RusimgError::FailedToReadFile(s) => write!(f, "Failed to read file: \n\t{}", s),
            RusimgError::FailedToGetMetadata(s) => write!(f, "Failed to get metadata: \n\t{}", s),
            RusimgError::FailedToOpenImage(s) => write!(f, "Failed to open image: \n\t{}", s),
            RusimgError::FailedToSaveImage(s) => write!(f, "Failed to save image: \n\t{}", s),
            RusimgError::FailedToCopyBinaryData(s) => write!(f, "Failed to copy binary data to memory: \n\t{}", s),
            RusimgError::FailedToGetFilename(s) => write!(f, "Failed to get filename: \n\t{}", s.display()),
            RusimgError::FailedToCreateFile(s) => write!(f, "Failed to create file: \n\t{}", s),
            RusimgError::FailedToWriteFIle(s) => write!(f, "Failed to write file: \n\t{}", s),
            RusimgError::FailedToDecodeWebp => write!(f, "Failed to decode webp"),
            RusimgError::FailedToEncodeWebp(s) => write!(f, "Failed to encode webp: \n\t{}", s),
            RusimgError::FailedToCompressImage(s) => {
                if let Some(s) = s {
                    write!(f, "Failed to compress image: \n\t{}", s)
                }
                else {
                    write!(f, "Failed to compress image")
                }
            }
            RusimgError::FailedToConvertPathToString => write!(f, "Failed to convert path to string"),
            RusimgError::InvalidTrimXY => write!(f, "Invalid trim XY"),
            RusimgError::ImageFormatCannotBeCompressed => write!(f, "this image format cannot be compressed"),
            RusimgError::UnsupportedFileExtension => write!(f, "Unsupported file extension"),
            RusimgError::ImageDataIsNone => write!(f, "Image data is None"),
            RusimgError::FailedToGetDynamicImage => write!(f, "Failed to get dynamic image"),
            RusimgError::FailedToConvertExtension => write!(f, "Failed to convert extension"),
        }
    }
}

/// RusImg object.
/// This object contains an image object and its metadata.
pub struct RusImg {
    pub extension: Extension,
    pub data: Box<(dyn RusimgTrait)>,
}

/// Rectangle object for rusimg.
/// This object is used for trimming an image.
#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// RusimgTrait is a trait for RusImg objects.
/// This trait is used for image operations.
/// Implement this trait for each image format.
pub trait RusimgTrait {
    /// Import an image from a DynamicImage object.
    fn import(image: DynamicImage, source_path: PathBuf, source_metadata: Metadata) -> Result<Self, RusimgError> where Self: Sized;
    /// Open an image from a image buffer.
    /// The ``path`` parameter is the file path of the image, but it is used for copying the file path to the object.
    /// This returns a RusImg object.
    fn open(path: PathBuf, image_buf: Vec<u8>, metadata: Metadata) -> Result<Self, RusimgError> where Self: Sized;
    /// Save the image to a file to the ``path``.
    fn save(&mut self, path: Option<PathBuf>) -> Result<(), RusimgError>;
    /// Compress the image with the quality parameter.
    fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError>;
    /// Resize the image with the resize_ratio parameter.
    fn resize(&mut self, resize_ratio: u8) -> Result<ImgSize, RusimgError>;
    /// Trim the image with the trim parameter.
    /// The trim parameter is a Rect object.
    fn trim(&mut self, trim: Rect) -> Result<ImgSize, RusimgError>;
    /// Grayscale the image.
    fn grayscale(&mut self);
    /// Set a image::DynamicImage to the image object.
    /// After setting the image, the image object will be updated.
    fn set_dynamic_image(&mut self, image: DynamicImage) -> Result<(), RusimgError>;
    /// Get a image::DynamicImage from the image object.
    fn get_dynamic_image(&mut self) -> Result<DynamicImage, RusimgError>;
    /// Get the source file path.
    fn get_source_filepath(&self) -> PathBuf;
    /// Get the destination file path.
    fn get_destination_filepath(&self) -> Option<PathBuf>;
    /// Get the source metadata.
    fn get_metadata_src(&self) -> Metadata;
    /// Get the destination metadata.
    fn get_metadata_dest(&self) -> Option<Metadata>;
    /// Get the image size.
    fn get_size(&self) -> ImgSize;

    /// Get a file path for saving an image.
    fn get_save_filepath(&self, source_filepath: &PathBuf, destination_filepath: Option<PathBuf>, new_extension: &String) -> Result<PathBuf, RusimgError> {
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

/// Image size object.
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

/// Save status object.
/// This object is used for tracking the status of saving an image.
/// It contains the output file path, the file size before saving, and the file size after saving.
/// If the image has compression, the file size after saving will be different from the file size before saving.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveStatus {
    pub output_path: Option<PathBuf>,
    pub before_filesize: u64,
    pub after_filesize: Option<u64>,
}

/// Image extension object.
/// By default, Rusimg supports BMP, JPEG, PNG, and WebP.
/// If you want to use another format, you can use ExternalFormat like this:
/// ```
/// let ext = Extension::ExternalFormat("tiff".to_string());
/// ```
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

// Get image format from image buffer.
fn guess_image_format(image_buf: &[u8]) -> Result<image::ImageFormat, RusimgError> {
    let format = image::guess_format(image_buf).map_err(|e| RusimgError::FailedToOpenImage(e.to_string()))?;
    Ok(format)
}

/// Open a bmp image file and make a RusImg object.
/// If the bmp feature is enabled, it will open a BMP image.
/// If not, it will return an UnsupportedFileExtension error.
#[cfg(feature="bmp")]
fn open_bmp_image(path: &Path, buf: Vec<u8>, metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    let image = bmp::BmpImage::open(path.to_path_buf(), buf, metadata_input)?;
    let data = Box::new(image);
    Ok(RusImg { extension: Extension::Bmp, data: data })
}
#[cfg(not(feature="bmp"))]
fn open_bmp_image(_path: &Path, _buf: Vec<u8>, _metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}
/// Open a jpeg image file and make a RusImg object.
/// If the jpeg feature is enabled, it will open a JPEG image.
/// If not, it will return an UnsupportedFileExtension error.
#[cfg(feature="jpeg")]
fn open_jpeg_image(path: &Path, buf: Vec<u8>, metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    let image = jpeg::JpegImage::open(path.to_path_buf(), buf, metadata_input)?;
    let data = Box::new(image);
    Ok(RusImg { extension: Extension::Jpeg, data: data })
}
#[cfg(not(feature="jpeg"))]
fn open_jpeg_image(_path: &Path, _buf: Vec<u8>, _metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}
/// Open a png image file and make a RusImg object.
/// If the png feature is enabled, it will open a PNG image.
/// If not, it will return an UnsupportedFileExtension error.
#[cfg(feature="png")]
fn open_png_image(path: &Path, buf: Vec<u8>, metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    let image = png::PngImage::open(path.to_path_buf(), buf, metadata_input)?;
    let data = Box::new(image);
    Ok(RusImg { extension: Extension::Png, data: data })
}
#[cfg(not(feature="png"))]
fn open_png_image(_path: &Path, _buf: Vec<u8>, _metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}
/// Open a webp image file and make a RusImg object.
/// If the webp feature is enabled, it will open a WebP image.
/// If not, it will return an UnsupportedFileExtension error.
#[cfg(feature="webp")]
fn open_webp_image(path: &Path, buf: Vec<u8>, metadata_input: Metadata) -> Result<RusImg, RusimgError> {
    let image = webp::WebpImage::open(path.to_path_buf(), buf, metadata_input)?;
    let data = Box::new(image);
    Ok(RusImg { extension: Extension::Webp, data: data })
}
#[cfg(not(feature="webp"))]
fn open_webp_image(_path: &Path, _buf: Vec<u8>, _metadata_input: Metadata) -> Result<RusImg, RusimgError> {
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

// Converter interfaces.
/// Convert a DynamicImage object to a BMP image object.
/// If the bmp feature is enabled, it will convert the DynamicImage to a BMP image.
/// If not, it will return an UnsupportedFileExtension error.
#[cfg(feature="bmp")]
fn convert_to_bmp_image(dynamic_image: DynamicImage, filepath: PathBuf, metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    let bmp = bmp::BmpImage::import(dynamic_image, filepath, metadata)?;
    Ok(Box::new(bmp))
}
#[cfg(not(feature="bmp"))]
fn convert_to_bmp_image(_dynamic_image: DynamicImage, _filepath: PathBuf, _metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}
/// Convert a DynamicImage object to a JPEG image object.
/// If the jpeg feature is enabled, it will convert the DynamicImage to a JPEG image.
/// If not, it will return an UnsupportedFileExtension error.
#[cfg(feature="jpeg")]
fn convert_to_jpeg_image(dynamic_image: DynamicImage, filepath: PathBuf, metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    let jpeg = jpeg::JpegImage::import(dynamic_image, filepath, metadata)?;
    Ok(Box::new(jpeg))
}
#[cfg(not(feature="jpeg"))]
fn convert_to_jpeg_image(_dynamic_image: DynamicImage, _filepath: PathBuf, _metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}
/// Convert a DynamicImage object to a PNG image object.
/// If the png feature is enabled, it will convert the DynamicImage to a PNG image.
/// If not, it will return an UnsupportedFileExtension error.
#[cfg(feature="png")]
fn convert_to_png_image(dynamic_image: DynamicImage, filepath: PathBuf, metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    let png = png::PngImage::import(dynamic_image, filepath, metadata)?;
    Ok(Box::new(png))
}
#[cfg(not(feature="png"))]
fn convert_to_png_image(_dynamic_image: DynamicImage, _filepath: PathBuf, _metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}
/// Convert a DynamicImage object to a WebP image object.
/// If the webp feature is enabled, it will convert the DynamicImage to a WebP image.
/// If not, it will return an UnsupportedFileExtension error.
#[cfg(feature="webp")]
fn convert_to_webp_image(dynamic_image: DynamicImage, filepath: PathBuf, metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    let webp = webp::WebpImage::import(dynamic_image, filepath, metadata)?;
    Ok(Box::new(webp))
}
#[cfg(not(feature="webp"))]
fn convert_to_webp_image(_dynamic_image: DynamicImage, _filepath: PathBuf, _metadata: Metadata) -> Result<Box<(dyn RusimgTrait)>, RusimgError> {
    Err(RusimgError::UnsupportedFileExtension)
}

/// RusImg object implementation.
/// The RusImg object wraps RusimgTrait functions.
impl RusImg {
    /// Get image size.
    /// This uses the ``get_size()`` function from ``RusimgTrait``.
    pub fn get_image_size(&self) -> Result<ImgSize, RusimgError> {
        let size = self.data.get_size();
        Ok(size)
    }

    /// Resize an image.
    /// It must be called after open_image().
    /// Set ratio to 100 to keep the original size.
    /// This uses the ``resize()`` function from ``RusimgTrait``.
    pub fn resize(&mut self, ratio: u8) -> Result<ImgSize, RusimgError> {
        let size = self.data.resize(ratio)?;
        Ok(size)
    }

    /// Trim an image. Set the trim area with four u32 values: x, y, w, h.
    /// It must be called after open_image().
    /// The values will be assigned to a Rect object.
    /// This uses the ``trim()`` function from ``RusimgTrait``.
    pub fn trim(&mut self, trim_x: u32, trim_y: u32, trim_w: u32, trim_h: u32) -> Result<ImgSize, RusimgError> {
        let size = self.data.trim(Rect{x: trim_x, y: trim_y, w: trim_w, h: trim_h})?;
        Ok(size)
    }
    /// Trim an image. Set the trim area with a rusimg::Rect object.
    /// It must be called after open_image().
    /// This uses the ``trim()`` function from ``RusimgTrait``.
    pub fn trim_rect(&mut self, trim_area: Rect) -> Result<ImgSize, RusimgError> {
        let size = self.data.trim(trim_area)?;
        Ok(size)
    }

    /// Grayscale an image.
    /// It must be called after open_image().
    /// This uses the ``grayscale()`` function from ``RusimgTrait``.
    pub fn grayscale(&mut self) -> Result<(), RusimgError> {
        self.data.grayscale();
        Ok(())
    }

    /// Compress an image.
    /// It must be called after open_image().
    /// Set quality to 100 to keep the original quality.
    /// This uses the ``compress()`` function from ``RusimgTrait``.
    pub fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError> {
        self.data.compress(quality)?;
        Ok(())
    }

    /// Convert an image to another format.
    /// And replace the original image with the new one.
    /// It must be called after open_image().
    /// This uses the ``get_dynamic_image()`` function to get the DynamicImage object, ``get_metadata_src()`` to get the metadata, and ``compress()`` to compress the image.
    pub fn convert(&mut self, new_extension: &Extension) -> Result<(), RusimgError> {
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

        self.extension = new_extension.clone();
        self.data = new_image;

        Ok(())
    }

    /// Set a ``image::DynamicImage`` to an RusImg.
    /// After setting the image, the image object will be updated.
    /// This uses the ``set_dynamic_image()`` function from ``RusimgTrait``.
    pub fn set_dynamic_image(&mut self, image: DynamicImage) -> Result<(), RusimgError> {
        self.data.set_dynamic_image(image)?;
        Ok(())
    }

    /// Get a ``image::DynamicImage`` from an RusImg.
    /// This uses the ``get_dynamic_image()`` function from ``RusimgTrait``.
    pub fn get_dynamic_image(&mut self) -> Result<DynamicImage, RusimgError> {
        let dynamic_image = self.data.get_dynamic_image()?;
        Ok(dynamic_image)
    }

    /// Get file extension.
    /// This returns the file extension of the image.
    pub fn get_extension(&self) -> Extension {
        self.extension.clone()
    }

    /// Get input file path.
    /// This returns the file path of the image.
    pub fn get_input_filepath(&self) -> PathBuf {
        self.data.get_source_filepath()
    }

    /// Save an image to a file.
    /// If path is None, the original file will be overwritten.
    /// This uses the ``get_destination_filepath()`` to get the destination file path, ``get_metadata_src()`` to get the source file size, and ``get_metadata_dest()`` to get the destination file size, and ``save()`` to save the image.
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
