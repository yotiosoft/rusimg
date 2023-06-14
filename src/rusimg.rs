mod bmp;
mod jpeg;
mod png;
mod webp;

use std::path::Path;
use std::fs::Metadata;
use std::fmt;
use image::DynamicImage;

#[derive(Debug, Clone, PartialEq)]
pub enum RusimgError {
    FailedToOpenFile(String),
    FailedToReadFile(String),
    FailedToGetMetadata(String),
    FailedToOpenImage(String),
    FailedToSaveImage(String),
    FailedToSaveImageInConverting,
    FailedToSaveImageInSaving,
    FailedToCopyBinaryData(String),
    FailedToGetFilename(String),
    FailedToCreateFile(String),
    FailedToWriteFIle(String),
    FailedToDecodeWebp,
    FailedToEncodeWebp(String),
    FailedToCompressImage(Option<String>),
    FailedToConvertFilenameToString,
    FailedToConvertPathToString,
    FailedToGetExtension,
    FailedToViewImage(String),
    InvalidTrimXY,
    BMPImagesCannotBeCompressed,
    UnsupportedFileExtension,
    ImageDataIsNone,
}
impl fmt::Display for RusimgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RusimgError::FailedToOpenFile(s) => write!(f, "Failed to open file: {}", s),
            RusimgError::FailedToReadFile(s) => write!(f, "Failed to read file: {}", s),
            RusimgError::FailedToGetMetadata(s) => write!(f, "Failed to get metadata: {}", s),
            RusimgError::FailedToOpenImage(s) => write!(f, "Failed to open image: {}", s),
            RusimgError::FailedToSaveImage(s) => write!(f, "Failed to save image: {}", s),
            RusimgError::FailedToSaveImageInConverting => write!(f, "Failed to save image"),
            RusimgError::FailedToSaveImageInSaving => write!(f, "Failed to save image"),
            RusimgError::FailedToCopyBinaryData(s) => write!(f, "Failed to copy binary data to memory: {}", s),
            RusimgError::FailedToGetFilename(s) => write!(f, "Failed to get filename: {}", s),
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
            RusimgError::FailedToConvertFilenameToString => write!(f, "Failed to convert filename to string"),
            RusimgError::FailedToConvertPathToString => write!(f, "Failed to convert path to string"),
            RusimgError::FailedToGetExtension => write!(f, "Failed to get extension"),
            RusimgError::FailedToViewImage(s) => write!(f, "Failed to view image: {}", s),
            RusimgError::InvalidTrimXY => write!(f, "Invalid trim XY"),
            RusimgError::BMPImagesCannotBeCompressed => write!(f, "BMP images cannot be compressed"),
            RusimgError::UnsupportedFileExtension => write!(f, "Unsupported file extension"),
            RusimgError::ImageDataIsNone => write!(f, "Image data is None"),
        }
    }
}

pub trait Rusimg {
    fn import(image: DynamicImage, source_path: String, source_metadata: Metadata) -> Result<Self, RusimgError> where Self: Sized;
    fn open(path: &str) -> Result<Self, RusimgError> where Self: Sized;
    fn save(&mut self, path: Option<&String>) -> Result<(), RusimgError>;
    fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError>;
    fn resize(&mut self, resize_ratio: u8) -> Result<(), RusimgError>;
    fn trim(&mut self, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<(), RusimgError>;
    fn grayscale(&mut self);
    fn view(&self) -> Result<(), RusimgError>;

    fn save_filepath(source_filepath: &String, destination_filepath: Option<&String>, new_extension: &String) -> Result<String, RusimgError> {
        if let Some(path) = destination_filepath {
            if Path::new(path).is_dir() {
                let filename = match Path::new(&source_filepath).file_name() {
                    Some(filename) => filename,
                    None => return Err(RusimgError::FailedToSaveImageInSaving),
                };
                match Path::new(path).join(filename).with_extension(new_extension).to_str() {
                    Some(s) => Ok(s.to_string()),
                    None => Err(RusimgError::FailedToSaveImageInSaving),
                }
            }
            else {
                Ok(path.to_string())
            }
        }
        else {
            match Path::new(&source_filepath).with_extension(new_extension).to_str() {
                Some(s) => Ok(s.to_string()),
                None => Err(RusimgError::FailedToSaveImageInConverting),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Extension {
    Bmp,
    Jpeg,
    Png,
    Webp,
}

#[derive(Debug, Clone)]
pub struct ImgData {
    bmp: Option<bmp::BmpImage>,
    jpeg: Option<jpeg::JpegImage>,
    png: Option<png::PngImage>,
    webp: Option<webp::WebpImage>,
}

#[derive(Debug, Clone)]
pub struct Img {
    pub extension: Extension,
    pub data: ImgData,
}

pub fn get_extension(path: &str) -> Result<Extension, RusimgError> {
    let path = path.to_ascii_lowercase();
    match Path::new(&path).extension().and_then(|s| s.to_str()) {
        Some("bmp") => Ok(Extension::Bmp),
        Some("jpg") | Some("jpeg") => Ok(Extension::Jpeg),
        Some("png") => Ok(Extension::Png),
        Some("webp") => Ok(Extension::Webp),
        _ => {
            if path.ends_with("bmp") {
                Ok(Extension::Bmp)
            } else if path.ends_with("jpg") || path.ends_with("jpeg") {
                Ok(Extension::Jpeg)
            } else if path.ends_with("png") {
                Ok(Extension::Png)
            } else if path.ends_with("webp") {
                Ok(Extension::Webp)
            } else {
                Err(RusimgError::UnsupportedFileExtension)
            }
        },
    }
}

pub fn open_image(path: &str) -> Result<Img, RusimgError> {
    match get_extension(&path) {
        Ok(Extension::Bmp) => {
            let bmp = bmp::BmpImage::open(&path)?;
            Ok(Img {
                extension: Extension::Bmp,
                data: ImgData {
                    bmp: Some(bmp),
                    jpeg: None,
                    png: None,
                    webp: None,
                },
            })
        },
        Ok(Extension::Jpeg) => {
            let jpeg = jpeg::JpegImage::open(&path)?;
            Ok(Img {
                extension: Extension::Jpeg,
                data: ImgData {
                    bmp: None,
                    jpeg: Some(jpeg),
                    png: None,
                    webp: None,
                },
            })
        },
        Ok(Extension::Png) => {
            let png = png::PngImage::open(&path)?;
            Ok(Img {
                extension: Extension::Png,
                data: ImgData {
                    bmp: None,
                    jpeg: None,
                    png: Some(png),
                    webp: None,
                },
            })
        },
        Ok(Extension::Webp) => {
            let webp = webp::WebpImage::open(&path)?;
            Ok(Img {
                extension: Extension::Webp,
                data: ImgData {
                    bmp: None,
                    jpeg: None,
                    png: None,
                    webp: Some(webp),
                },
            })
        },
        Err(e) => Err(e),
    }
}

pub fn resize(source_image: &mut Img, resize_ratio: u8) -> Result<(), RusimgError> {
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

pub fn trim(image: &mut Img, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<(), RusimgError> {
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

pub fn grayscale(image: &mut Img) {
    match image.extension {
        Extension::Bmp => {
            match &mut image.data.bmp {
                Some(bmp) => {
                    bmp.grayscale()
                },
                None => (),
            }
        },
        Extension::Jpeg => {
            match &mut image.data.jpeg {
                Some(jpeg) => {
                    jpeg.grayscale()
                },
                None => (),
            }
        },
        Extension::Png => {
            match &mut image.data.png {
                Some(png) => {
                    png.grayscale()
                },
                None => (),
            }
        },
        Extension::Webp => {
            match &mut image.data.webp {
                Some(webp) => {
                    webp.grayscale()
                },
                None => (),
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
                None => return Err(RusimgError::FailedToSaveImageInSaving),
            }
        },
        Extension::Jpeg => {
            match &mut data.jpeg {
                Some(jpeg) => {
                    jpeg.compress(quality)
                },
                None => return Err(RusimgError::FailedToSaveImageInSaving),
            }
        },
        Extension::Png => {
            match &mut data.png {
                Some(png) => {
                    png.compress(quality)
                },
                None => return Err(RusimgError::FailedToSaveImageInSaving),
            }
        },
        Extension::Webp => {
            match &mut data.webp {
                Some(webp) => {
                    webp.compress(quality)
                },
                None => return Err(RusimgError::FailedToSaveImageInSaving),
            }
        },
    }
}

pub fn convert(source_img: &mut Img, destination_extension: &Extension) -> Result<Img, RusimgError> {
    match source_img.extension {
        Extension::Bmp => {
            match &source_img.data.bmp {
                Some(bmp) => {
                    let dynamic_image = bmp.image.clone();
                    match destination_extension {
                        Extension::Bmp => {
                            Ok(source_img.clone())
                        },
                        Extension::Jpeg => {
                            let jpeg = jpeg::JpegImage::import(dynamic_image, bmp.filepath_input.clone(), bmp.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Jpeg,
                                data: ImgData {
                                    bmp: None,
                                    jpeg: Some(jpeg),
                                    png: None,
                                    webp: None,
                                },
                            })
                        }
                        Extension::Png => {
                            let png = png::PngImage::import(dynamic_image, bmp.filepath_input.clone(), bmp.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Png,
                                data: ImgData {
                                    bmp: None,
                                    jpeg: None,
                                    png: Some(png),
                                    webp: None,
                                },
                            })
                        },
                        Extension::Webp => {
                            let webp = webp::WebpImage::import(dynamic_image, bmp.filepath_input.clone(), bmp.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Webp,
                                data: ImgData {
                                    bmp: None,
                                    jpeg: None,
                                    png: None,
                                    webp: Some(webp),
                                },
                            })
                        },
                    }
                },
                None => return Err(RusimgError::FailedToSaveImageInConverting),
            }
        },
        Extension::Jpeg => {
            match &source_img.data.jpeg {
                Some(jpeg) => {
                    let dynamic_image = jpeg.image.clone();
                    match destination_extension {
                        Extension::Bmp => {
                            let bmp = bmp::BmpImage::import(dynamic_image, jpeg.filepath_input.clone(), jpeg.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Bmp,
                                data: ImgData {
                                    bmp: Some(bmp),
                                    jpeg: None,
                                    png: None,
                                    webp: None,
                                },
                            })
                        },
                        Extension::Jpeg => {
                            Ok(source_img.clone())
                        },
                        Extension::Png => {
                            let png = png::PngImage::import(dynamic_image, jpeg.filepath_input.clone(), jpeg.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Png,
                                data: ImgData {
                                    bmp: None,
                                    jpeg: None,
                                    png: Some(png),
                                    webp: None,
                                },
                            })
                        },
                        Extension::Webp => {
                            let webp = webp::WebpImage::import(dynamic_image, jpeg.filepath_input.clone(), jpeg.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Webp,
                                data: ImgData {
                                    bmp: None,
                                    jpeg: None,
                                    png: None,
                                    webp: Some(webp),
                                },
                            })
                        },
                    }
                },
                None => return Err(RusimgError::FailedToSaveImageInConverting),
            }
        },
        Extension::Png => {
            match &source_img.data.png {
                Some(png) => {
                    let dynamic_image = png.image.clone();
                    match destination_extension {
                        Extension::Bmp => {
                            let bmp = bmp::BmpImage::import(dynamic_image, png.filepath_input.clone(), png.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Bmp,
                                data: ImgData {
                                    bmp: Some(bmp),
                                    jpeg: None,
                                    png: None,
                                    webp: None,
                                },
                            })
                        },
                        Extension::Jpeg => {
                            let jpeg = jpeg::JpegImage::import(dynamic_image, png.filepath_input.clone(), png.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Jpeg,
                                data: ImgData {
                                    bmp: None,
                                    jpeg: Some(jpeg),
                                    png: None,
                                    webp: None,
                                },
                            })
                        },
                        Extension::Png => {
                            Ok(source_img.clone())
                        },
                        Extension::Webp => {
                            let webp = webp::WebpImage::import(dynamic_image, png.filepath_input.clone(), png.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Webp,
                                data: ImgData {
                                    bmp: None,
                                    jpeg: None,
                                    png: None,
                                    webp: Some(webp),
                                },
                            })
                        },
                    }
                },
                None => return Err(RusimgError::FailedToSaveImageInConverting),
            }
        },
        Extension::Webp => {
            match &source_img.data.webp {
                Some(webp) => {
                    let dynamic_image = webp.image.clone();
                    match destination_extension {
                        Extension::Bmp => {
                            let bmp = bmp::BmpImage::import(dynamic_image, webp.filepath_input.clone(), webp.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Bmp,
                                data: ImgData {
                                    bmp: Some(bmp),
                                    jpeg: None,
                                    png: None,
                                    webp: None,
                                },
                            })
                        },
                        Extension::Jpeg => {
                            let jpeg = jpeg::JpegImage::import(dynamic_image, webp.filepath_input.clone(), webp.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Jpeg,
                                data: ImgData {
                                    bmp: None,
                                    jpeg: Some(jpeg),
                                    png: None,
                                    webp: None,
                                },
                            })
                        },
                        Extension::Png => {
                            let png = png::PngImage::import(dynamic_image, webp.filepath_input.clone(), webp.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Png,
                                data: ImgData {
                                    bmp: None,
                                    jpeg: None,
                                    png: Some(png),
                                    webp: None,
                                },
                            })
                        },
                        Extension::Webp => {
                            Ok(source_img.clone())
                        },
                    }
                },
                None => return Err(RusimgError::FailedToSaveImageInConverting),
            }
        },
    }
}

pub fn save_print(before_path: &String, after_path: &String, before_size: u64, after_size: u64) {
    if before_path == after_path {
        println!("Overwrite: {}", before_path);
        println!("File Size: {} -> {} ({:.1}%)", before_size, after_size, (after_size as f64 / before_size as f64) * 100.0);
    }
    else if get_extension(before_path) != get_extension(after_path) {
        println!("Convert: {} -> {}", before_path, after_path);
        println!("File Size: {} -> {} ({:.1}%)", before_size, after_size, (after_size as f64 / before_size as f64) * 100.0);
    }
    else {
        println!("Move: {} -> {}", before_path, after_path);
        println!("File Size: {} -> {} ({:.1}%)", before_size, after_size, (after_size as f64 / before_size as f64) * 100.0);
    }
}

pub fn save_image(path: Option<&String>, data: &mut ImgData, extension: &Extension) -> Result<String, RusimgError> {
    match extension {
        Extension::Bmp => {
            match data.bmp {
                Some(ref mut bmp) => {
                    bmp.save(path)?;
                    save_print(
                        &bmp.filepath_input, &bmp.filepath_output.as_ref().unwrap(), 
                        bmp.metadata_input.len(), bmp.metadata_output.as_ref().unwrap().len()
                    );
                    Ok(bmp.filepath_output.as_deref().unwrap().to_string())
                },
                None => return Err(RusimgError::FailedToSaveImageInSaving),
            }
        },
        Extension::Jpeg => {
            match data.jpeg {
                Some(ref mut jpeg) => {
                    jpeg.save(path)?;
                    save_print(
                        &jpeg.filepath_input, &jpeg.filepath_output.as_ref().unwrap(), 
                        jpeg.metadata_input.len(), jpeg.metadata_output.as_ref().unwrap().len()
                    );
                    Ok(jpeg.filepath_output.as_deref().unwrap().to_string())
                },
                None => return Err(RusimgError::FailedToSaveImageInSaving),
            }
        },
        Extension::Png => {
            match data.png {
                Some(ref mut png) => {
                    png.save(path)?;
                    save_print(
                        &png.filepath_input, &png.filepath_output.as_ref().unwrap(), 
                        png.metadata_input.len(), png.metadata_output.as_ref().unwrap().len()
                    );
                    Ok(png.filepath_output.as_deref().unwrap().to_string())
                },
                None => return Err(RusimgError::FailedToSaveImageInSaving),
            }
        },
        Extension::Webp => {
            match data.webp {
                Some(ref mut webp) => {
                    webp.save(path)?;
                    save_print(
                        &webp.filepath_input, &webp.filepath_output.as_ref().unwrap(), 
                        webp.metadata_input.len(), webp.metadata_output.as_ref().unwrap().len()
                    );
                    Ok(webp.filepath_output.as_deref().unwrap().to_string())
                },
                None => return Err(RusimgError::FailedToSaveImageInSaving),
            }
        },
    }
}

pub fn view(image: &mut Img) -> Result<(), RusimgError> {
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
