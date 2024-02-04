pub mod bmp;
pub mod jpeg;
pub mod png;
pub mod webp;

use std::path::{Path, PathBuf};
use image::{ImageFormat, DynamicImage};
use std::fs::Metadata;
use std::io::Read;
use super::{RusImg, ImgSize, ImgData, RusimgError, RusimgStatus, Extension, SaveStatus};

pub trait RusimgTrait {
    fn import(image: DynamicImage, source_path: PathBuf, source_metadata: Metadata) -> Result<Self, RusimgError> where Self: Sized;
    fn open(path: PathBuf, image_buf: Vec<u8>, metadata: Metadata) -> Result<Self, RusimgError> where Self: Sized;
    fn save(&mut self, path: Option<PathBuf>) -> Result<RusimgStatus, RusimgError>;
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
}

// 画像フォーマットを取得
fn do_guess_image_format(image_buf: &[u8]) -> Result<image::ImageFormat, RusimgError> {
    let format = image::guess_format(image_buf).map_err(|e| RusimgError::FailedToOpenImage(e.to_string()))?;
    Ok(format)
}

// 画像サイズを取得
pub fn do_get_image_size(img: &RusImg) -> Result<ImgSize, RusimgError> {
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

pub fn do_open_image(path: &Path) -> Result<RusImg, RusimgError> {
    let mut raw_data = std::fs::File::open(&path.to_path_buf()).map_err(|e| RusimgError::FailedToOpenFile(e.to_string()))?;
    let mut buf = Vec::new();
    raw_data.read_to_end(&mut buf).map_err(|e| RusimgError::FailedToReadFile(e.to_string()))?;
    let metadata_input = raw_data.metadata().map_err(|e| RusimgError::FailedToGetMetadata(e.to_string()))?;

    match do_guess_image_format(&buf)? {
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

pub fn do_resize(source_image: &mut RusImg, resize_ratio: u8) -> Result<ImgSize, RusimgError> {
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

pub fn do_trim(image: &mut RusImg, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<ImgSize, RusimgError> {
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

pub fn do_grayscale(image: &mut RusImg) -> Result<(), RusimgError> {
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

pub fn do_compress(data: &mut ImgData, extension: &Extension, quality: Option<f32>) -> Result<(), RusimgError> {
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

pub fn do_convert(original: &mut RusImg, to: &Extension) -> Result<RusImg, RusimgError> {
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

pub fn do_save_image(path: Option<PathBuf>, data: &mut ImgData, extension: &Extension) -> Result<SaveStatus, RusimgError> {
    match extension {
        Extension::Bmp => {
            match data.bmp {
                Some(ref mut bmp) => {
                    let status = bmp.save(path)?;
                    let ret = SaveStatus {
                        status: status, 
                        output_path: bmp.filepath_output.clone().or(None),
                        before_filesize: bmp.metadata_input.len(), 
                        after_filesize: bmp.metadata_output.as_ref().or(None).map(|m| m.len())
                    };
                    Ok(ret)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Jpeg => {
            match data.jpeg {
                Some(ref mut jpeg) => {
                    let status = jpeg.save(path)?;
                    let ret = SaveStatus {
                        status: status, 
                        output_path: jpeg.filepath_output.clone().or(None),
                        before_filesize: jpeg.metadata_input.len(), 
                        after_filesize: jpeg.metadata_output.as_ref().or(None).map(|m| m.len())
                    };
                    Ok(ret)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Png => {
            match data.png {
                Some(ref mut png) => {
                    let status = png.save(path)?;
                    let ret = SaveStatus {
                        status: status, 
                        output_path: png.filepath_output.clone().or(None),
                        before_filesize: png.metadata_input.len(), 
                        after_filesize: png.metadata_output.as_ref().or(None).map(|m| m.len())
                    };
                    Ok(ret)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
        Extension::Webp => {
            match data.webp {
                Some(ref mut webp) => {
                    let status = webp.save(path)?;
                    let ret = SaveStatus {
                        status: status, 
                        output_path: webp.filepath_output.clone().or(None),
                        before_filesize: webp.metadata_input.len(), 
                        after_filesize: webp.metadata_output.as_ref().or(None).map(|m| m.len())
                    };
                    Ok(ret)
                },
                None => return Err(RusimgError::ImageDataIsNone),
            }
        },
    }
}

pub fn do_view(image: &mut RusImg) -> Result<(), RusimgError> {
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

