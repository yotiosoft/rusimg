pub mod bmp;
pub mod jpeg;
pub mod png;
pub mod webp;

use std::path::{Path, PathBuf};
use image::{ImageFormat, DynamicImage};
use std::fs::Metadata;
use std::io::Read;
use super::{RusImg, ImgSize, RusimgError, Extension, SaveStatus};

pub struct ImgData {
    pub image_struct: (dyn RusimgTrait),
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

// 画像フォーマットを取得
fn do_guess_image_format(image_buf: &[u8]) -> Result<image::ImageFormat, RusimgError> {
    let format = image::guess_format(image_buf).map_err(|e| RusimgError::FailedToOpenImage(e.to_string()))?;
    Ok(format)
}

// 画像サイズを取得
pub fn do_get_image_size(img: &RusImg) -> Result<ImgSize, RusimgError> {
    let w = img.data.image_struct.image.width() as usize;
    let h = img.data.image_struct.image.height() as usize;
    Ok(ImgSize::new(w, h))
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
                data: ImgData { image_struct: bmp },
            })
        },
        ImageFormat::Jpeg => {
            let jpeg = jpeg::JpegImage::open(path.to_path_buf(), buf, metadata_input)?;
            Ok(RusImg {
                extension: Extension::Jpeg,
                data: ImgData { image_struct: jpeg },
            })
        },
        ImageFormat::Png => {
            let png = png::PngImage::open(path.to_path_buf(), buf, metadata_input)?;
            Ok(RusImg {
                extension: Extension::Png,
                data: ImgData { image_struct: png },
            })
        },
        ImageFormat::WebP => {
            let webp = webp::WebpImage::open(path.to_path_buf(), buf, metadata_input)?;
            Ok(RusImg {
                extension: Extension::Webp,
                data: ImgData { image_struct: webp },
            })
        },
        _ => Err(RusimgError::UnsupportedFileExtension),
    }
}

pub fn do_resize(source_image: &mut RusImg, resize_ratio: u8) -> Result<ImgSize, RusimgError> {
    source_image.data.image_struct.resize(resize_ratio)
}

pub fn do_trim(image: &mut RusImg, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<ImgSize, RusimgError> {
    image.data.image_struct.trim(trim_xy, trim_wh)
}

pub fn do_grayscale(image: &mut RusImg) {
    image.data.image_struct.grayscale()
}

pub fn do_compress(data: &mut ImgData, extension: &Extension, quality: Option<f32>) -> Result<(), RusimgError> {
    data.image_struct.compress(quality)
}

pub fn do_convert(original: &mut RusImg, to: &Extension) -> Result<RusImg, RusimgError> {
    let dynamic_image = original.data.image_struct.image.clone();
    let filepath = original.data.image_struct.filepath_input.clone();
    let metadata = original.data.image_struct.metadata_input.clone();

    let new_image = match to {
        Extension::Bmp => {
            let bmp = bmp::BmpImage::import(dynamic_image, filepath, metadata)?;
            ImgData { image_struct: bmp }
        },
        Extension::Jpeg => {
            let jpeg = jpeg::JpegImage::import(dynamic_image, filepath, metadata)?;
            ImgData { image_struct: jpeg }
        },
        Extension::Png => {
            let png = png::PngImage::import(dynamic_image, filepath, metadata)?;
            ImgData { image_struct: png }
        },
        Extension::Webp => {
            let webp = webp::WebpImage::import(dynamic_image, filepath, metadata)?;
            ImgData { image_struct: webp }
        },
    };

    let extension = to.clone();

    Ok(RusImg {
        extension,
        data: new_image,
    })
}

pub fn do_save_image(path: Option<PathBuf>, data: &mut ImgData, extension: &Extension) -> Result<SaveStatus, RusimgError> {
    data.image_struct.save(path)?;
    let ret = SaveStatus {
        output_path: data.image_struct.filepath_output.clone().or(None),
        before_filesize: data.image_struct.metadata_input.len(), 
        after_filesize: data.image_struct.metadata_output.as_ref().or(None).map(|m| m.len())
    };
    Ok(ret)
}

pub fn do_view(image: &mut RusImg) -> Result<(), RusimgError> {
    image.data.image_struct.view()
}

