mod bmp;
mod jpeg;
mod png;
mod webp;

use std::path::Path;
use image::DynamicImage;
use std::fs::Metadata;

pub trait Rusimg {
    fn import(image: DynamicImage, source_path: String, source_metadata: Metadata) -> Result<Self, String> where Self: Sized;
    fn open(path: &str) -> Result<Self, String> where Self: Sized;
    fn save(&mut self, path: Option<&String>) -> Result<(), String>;
    fn compress(&mut self, quality: Option<f32>) -> Result<(), String>;
    fn resize(&mut self, resize_ratio: u8) -> Result<(), String>;
    fn view(&self) -> Result<(), String>;

    fn save_filepath(source_filepath: &String, destination_filepath: Option<&String>, new_extension: &String) -> String {
        if let Some(path) = destination_filepath {
            if Path::new(path).is_dir() {
                let filename = Path::new(&source_filepath).file_name().unwrap().to_str().unwrap();
                Path::new(path).join(filename).with_extension(new_extension).to_str().unwrap().to_string()
            }
            else {
                path.to_string()
            }
        }
        else {
            Path::new(&source_filepath).with_extension(new_extension).to_str().unwrap().to_string()
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

pub fn get_extension(path: &str) -> Result<Extension, String> {
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
                Err("Unsupported file extension".to_string())
            }
        },
    }
}

pub fn open_image(path: &str) -> Result<Img, String> {
    match get_extension(&path) {
        Ok(Extension::Bmp) => {
            let bmp = bmp::BmpImage::open(&path).map_err(|e| e.to_string())?;
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
            let jpeg = jpeg::JpegImage::open(&path).map_err(|e| e.to_string())?;
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
            let png = png::PngImage::open(&path).map_err(|e| e.to_string())?;
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
            let webp = webp::WebpImage::open(&path).map_err(|e| e.to_string())?;
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

pub fn resize(source_image: &mut Img, resize_ratio: u8) -> Result<(), String> {
    match source_image.extension {
        Extension::Bmp => {
            match &mut source_image.data.bmp {
                Some(bmp) => {
                    bmp.resize(resize_ratio)
                },
                None => return Err("Failed to save bmp image".to_string()),
            }
        },
        Extension::Jpeg => {
            match &mut source_image.data.jpeg {
                Some(jpeg) => {
                    jpeg.resize(resize_ratio)
                },
                None => return Err("Failed to save jpeg image".to_string()),
            }
        },
        Extension::Png => {
            match &mut source_image.data.png {
                Some(png) => {
                    png.resize(resize_ratio)
                },
                None => return Err("Failed to save png image".to_string()),
            }
        },
        Extension::Webp => {
            match &mut source_image.data.webp {
                Some(webp) => {
                    webp.resize(resize_ratio)
                },
                None => return Err("Failed to save webp image".to_string()),
            }
        },
    }
}

pub fn compress(data: &mut ImgData, extension: &Extension, quality: Option<f32>) -> Result<(), String> {
    match extension {
        Extension::Bmp => {
            match &mut data.bmp {
                Some(bmp) => {
                    bmp.compress(quality)
                },
                None => return Err("Failed to save bmp image".to_string()),
            }
        },
        Extension::Jpeg => {
            match &mut data.jpeg {
                Some(jpeg) => {
                    jpeg.compress(quality)
                },
                None => return Err("Failed to save jpeg image".to_string()),
            }
        },
        Extension::Png => {
            match &mut data.png {
                Some(png) => {
                    png.compress(quality)
                },
                None => return Err("Failed to save png image".to_string()),
            }
        },
        Extension::Webp => {
            match &mut data.webp {
                Some(webp) => {
                    webp.compress(quality)
                },
                None => return Err("Failed to save webp image".to_string()),
            }
        },
    }
}

pub fn convert(source_img: &mut Img, destination_extension: &Extension) -> Result<Img, String> {
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
                None => return Err("Failed to save jpeg image".to_string()),
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
                None => return Err("Failed to save jpeg image".to_string()),
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
                None => return Err("Failed to save png image".to_string()),
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
                None => return Err("Failed to save webp image".to_string()),
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

pub fn save_image(path: Option<&String>, data: &mut ImgData, extension: &Extension) -> Result<String, String> {
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
                None => return Err("Failed to save bmp image".to_string()),
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
                None => return Err("Failed to save jpeg image".to_string()),
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
                None => return Err("Failed to save png image".to_string()),
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
                None => return Err("Failed to save webp image".to_string()),
            }
        },
    }
}

pub fn view(image: &mut Img) -> Result<(), String> {
    match image.extension {
        Extension::Bmp => {
            match &mut image.data.bmp {
                Some(bmp) => {
                    bmp.view()
                },
                None => return Err("Failed to view bmp image".to_string()),
            }
        },
        Extension::Jpeg => {
            match &mut image.data.jpeg {
                Some(jpeg) => {
                    jpeg.view()
                },
                None => return Err("Failed to view jpeg image".to_string()),
            }
        },
        Extension::Png => {
            match &mut image.data.png {
                Some(png) => {
                    png.view()
                },
                None => return Err("Failed to view png image".to_string()),
            }
        },
        Extension::Webp => {
            match &mut image.data.webp {
                Some(webp) => {
                    webp.view()
                },
                None => return Err("Failed to view webp image".to_string()),
            }
        },
    }
}
