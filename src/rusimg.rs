mod bmp;
mod jpeg;
mod png;
mod webp;

use std::path::{Path, PathBuf};
use std::fs::Metadata;
use std::fmt;
use image::{DynamicImage, ImageFormat};
use std::io::Read;
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
}

pub trait Rusimg {
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

#[derive(Debug, Clone, Default)]
pub struct ImgData {
    pub bmp: Option<bmp::BmpImage>,
    pub jpeg: Option<jpeg::JpegImage>,
    pub png: Option<png::PngImage>,
    pub webp: Option<webp::WebpImage>,
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

// 画像フォーマットを取得
fn guess_image_format(image_buf: &[u8]) -> Result<image::ImageFormat, RusimgError> {
    let format = image::guess_format(image_buf).map_err(|e| RusimgError::FailedToOpenImage(e.to_string()))?;
    Ok(format)
}

// 画像サイズを取得
pub fn get_image_size(img: &RusImg) -> Result<ImgSize, RusimgError> {
    match img.extension {
        Extension::Bmp => {
            if img.data.bmp.is_none() {
                return Err(RusimgError::FailedToGetDynamicImage);
            }
            let w = img.data.bmp.as_ref().unwrap().image.width() as usize;
            let h = img.data.bmp.as_ref().unwrap().image.height() as usize;
            Ok(ImgSize::new(w, h))
        }
        Extension::Jpeg => {
            if img.data.jpeg.is_none() {
                return Err(RusimgError::FailedToGetDynamicImage);
            }
            let w = img.data.jpeg.as_ref().unwrap().image.width() as usize;
            let h = img.data.jpeg.as_ref().unwrap().image.height() as usize;
            Ok(ImgSize::new(w, h))
        }
        Extension::Png => {
            if img.data.png.is_none() {
                return Err(RusimgError::FailedToGetDynamicImage);
            }
            let w = img.data.png.as_ref().unwrap().image.width() as usize;
            let h = img.data.png.as_ref().unwrap().image.height() as usize;
            Ok(ImgSize::new(w, h))
        }
        Extension::Webp => {
            if img.data.webp.is_none() {
                return Err(RusimgError::FailedToGetDynamicImage);
            }
            let w = img.data.webp.as_ref().unwrap().image.width() as usize;
            let h = img.data.webp.as_ref().unwrap().image.height() as usize;
            Ok(ImgSize::new(w, h))
        }
    }
}

pub fn open_image(path: &Path) -> Result<RusImg, RusimgError> {
    let mut raw_data = std::fs::File::open(&path.to_path_buf()).map_err(|e| RusimgError::FailedToOpenFile(e.to_string()))?;
    let mut buf = Vec::new();
    raw_data.read_to_end(&mut buf).map_err(|e| RusimgError::FailedToReadFile(e.to_string()))?;
    let metadata_input = raw_data.metadata().map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?;

    match guess_image_format(&buf)? {
        ImageFormat::Bmp => {
            let bmp = bmp::BmpImage::open(path.to_path_buf(), buf, metadata_input)?;
            Ok(RusImg {
                extension: Extension::Bmp,
                data: ImgData { bmp: Some(bmp), ..Default::default() },
            })
        },
        ImageFormat::Jpeg => {
            let jpeg = jpeg::JpegImage::open(path.to_path_buf(), buf, metadata_input)?;
            Ok(RusImg {
                extension: Extension::Jpeg,
                data: ImgData { jpeg: Some(jpeg), ..Default::default() },
            })
        },
        ImageFormat::Png => {
            let png = png::PngImage::open(path.to_path_buf(), buf, metadata_input)?;
            Ok(RusImg {
                extension: Extension::Png,
                data: ImgData { png: Some(png), ..Default::default() },
            })
        },
        ImageFormat::WebP => {
            let webp = webp::WebpImage::open(path.to_path_buf(), buf, metadata_input)?;
            Ok(RusImg {
                extension: Extension::Webp,
                data: ImgData { webp: Some(webp), ..Default::default() },
            })
        },
        _ => Err(RusimgError::UnsupportedFileExtension),
    }
}

pub fn resize(source_image: &mut RusImg, resize_ratio: u8) -> Result<ImgSize, RusimgError> {
    match source_image.extension {
        Extension::Bmp => {
            match &mut source_image.data.bmp {
                Some(bmp) => {
                    bmp.resize(resize_ratio)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Jpeg => {
            match &mut source_image.data.jpeg {
                Some(jpeg) => {
                    jpeg.resize(resize_ratio)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Png => {
            match &mut source_image.data.png {
                Some(png) => {
                    png.resize(resize_ratio)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Webp => {
            match &mut source_image.data.webp {
                Some(webp) => {
                    webp.resize(resize_ratio)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
    }
}

pub fn trim(image: &mut RusImg, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<ImgSize, RusimgError> {
    match image.extension {
        Extension::Bmp => {
            match &mut image.data.bmp {
                Some(bmp) => {
                    bmp.trim(trim_xy, trim_wh)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Jpeg => {
            match &mut image.data.jpeg {
                Some(jpeg) => {
                    jpeg.trim(trim_xy, trim_wh)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Png => {
            match &mut image.data.png {
                Some(png) => {
                    png.trim(trim_xy, trim_wh)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Webp => {
            match &mut image.data.webp {
                Some(webp) => {
                    webp.trim(trim_xy, trim_wh)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
    }
}

pub fn grayscale(image: &mut RusImg) -> Result<(), RusimgError> {
    match image.extension {
        Extension::Bmp => {
            match &mut image.data.bmp {
                Some(bmp) => {
                    bmp.grayscale();
                    Ok(())
                },
                None => Err(RusimgError::ImageDataIsNone)
            }
        },
        Extension::Jpeg => {
            match &mut image.data.jpeg {
                Some(jpeg) => {
                    jpeg.grayscale();
                    Ok(())
                },
                None => Err(RusimgError::ImageDataIsNone)
            }
        },
        Extension::Png => {
            match &mut image.data.png {
                Some(png) => {
                    png.grayscale();
                    Ok(())
                },
                None => Err(RusimgError::ImageDataIsNone)
            }
        },
        Extension::Webp => {
            match &mut image.data.webp {
                Some(webp) => {
                    webp.grayscale();
                    Ok(())
                },
                None => Err(RusimgError::ImageDataIsNone)
            }
        },
    }
}

pub fn compress(data: &mut ImgData, extension: &Extension, quality: Option<f32>) -> Result<(), RusimgError> {
    match extension {
        Extension::Bmp => {
            match &mut data.bmp {
                Some(bmp) => {
                    bmp.compress(quality)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Jpeg => {
            match &mut data.jpeg {
                Some(jpeg) => {
                    jpeg.compress(quality)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Png => {
            match &mut data.png {
                Some(png) => {
                    png.compress(quality)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Webp => {
            match &mut data.webp {
                Some(webp) => {
                    webp.compress(quality)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
    }
}

pub fn convert(original: &mut RusImg, to: &Extension) -> Result<RusImg, RusimgError> {
    let (dynamic_image, filepath, metadata) = match original.extension {
        Extension::Bmp => {
            match &original.data.bmp {
                Some(bmp) => (bmp.image.clone(), bmp.filepath_input.clone(), bmp.metadata_input.clone()),
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Jpeg => {
            match &original.data.jpeg {
                Some(jpeg) => (jpeg.image.clone(), jpeg.filepath_input.clone(), jpeg.metadata_input.clone()),
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Png => {
            match &original.data.png {
                Some(png) => (png.image.clone(), png.filepath_input.clone(), png.metadata_input.clone()),
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Webp => {
            match &original.data.webp {
                Some(webp) => (webp.image.clone(), webp.filepath_input.clone(), webp.metadata_input.clone()),
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
    };

    match to {
        Extension::Bmp => {
            let bmp = bmp::BmpImage::import(dynamic_image, filepath, metadata)?;
            Ok(RusImg {
                extension: Extension::Bmp,
                data: ImgData { bmp: Some(bmp), ..Default::default() },
            })
        },
        Extension::Jpeg => {
            let jpeg = jpeg::JpegImage::import(dynamic_image, filepath, metadata)?;
            Ok(RusImg {
                extension: Extension::Jpeg,
                data: ImgData { jpeg: Some(jpeg), ..Default::default() },
            })
        },
        Extension::Png => {
            let png = png::PngImage::import(dynamic_image, filepath, metadata)?;
            Ok(RusImg {
                extension: Extension::Png,
                data: ImgData { png: Some(png), ..Default::default() },
            })
        },
        Extension::Webp => {
            let webp = webp::WebpImage::import(dynamic_image, filepath, metadata)?;
            Ok(RusImg {
                extension: Extension::Webp,
                data: ImgData { webp: Some(webp), ..Default::default() },
            })
        },
    }
}

pub fn save_image(path: Option<PathBuf>, data: &mut ImgData, extension: &Extension, file_overwrite_ask: FileOverwriteAsk) -> Result<(RusimgStatus, Option<PathBuf>, PathBuf, u64, Option<u64>), RusimgError> {
    match extension {
        Extension::Bmp => {
            match data.bmp {
                Some(ref mut bmp) => {
                    let status = bmp.save(path, &file_overwrite_ask)?;
                    Ok((status,
                        //bmp.filepath_output.clone().unwrap(), 
                        bmp.filepath_output.clone().or(None),
                        bmp.filepath_input.clone(), 
                        bmp.metadata_input.len(), 
                        bmp.metadata_output.as_ref().or(None).map(|m| m.len())))
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Jpeg => {
            match data.jpeg {
                Some(ref mut jpeg) => {
                    let status = jpeg.save(path, &file_overwrite_ask)?;
                    Ok((status,
                        jpeg.filepath_output.clone().or(None),
                        jpeg.filepath_input.clone(), 
                        jpeg.metadata_input.len(), 
                        jpeg.metadata_output.as_ref().or(None).map(|m| m.len())))
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Png => {
            match data.png {
                Some(ref mut png) => {
                    let status = png.save(path, &file_overwrite_ask)?;
                    Ok((status,
                        png.filepath_output.clone().or(None),
                        png.filepath_input.clone(), 
                        png.metadata_input.len(), 
                        png.metadata_output.as_ref().or(None).map(|m| m.len())))
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Webp => {
            match data.webp {
                Some(ref mut webp) => {
                    let status = webp.save(path, &file_overwrite_ask)?;
                    Ok((status,
                        webp.filepath_output.clone().or(None),
                        webp.filepath_input.clone(), 
                        webp.metadata_input.len(), 
                        webp.metadata_output.as_ref().or(None).map(|m| m.len())))
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
    }
}

pub fn view(image: &mut RusImg) -> Result<(), RusimgError> {
    match image.extension {
        Extension::Bmp => {
            match &mut image.data.bmp {
                Some(bmp) => {
                    bmp.view()
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Jpeg => {
            match &mut image.data.jpeg {
                Some(jpeg) => {
                    jpeg.view()
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Png => {
            match &mut image.data.png {
                Some(png) => {
                    png.view()
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Webp => {
            match &mut image.data.webp {
                Some(webp) => {
                    webp.view()
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
    }
}
