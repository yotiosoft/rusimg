mod jpeg;
mod png;
mod webp;

use std::path::Path;
use image::DynamicImage;
use std::fs::Metadata;

pub trait Rusimg {
    fn import(image: DynamicImage, source_path: String, source_metadata: Metadata) -> Result<Self, String> where Self: Sized;
    fn open(path: &str) -> Result<Self, String> where Self: Sized;
    fn save(&mut self, path: Option<&String>, quality: Option<f32>) -> Result<(), String>;
    fn compress(&mut self) -> Result<(), String>;

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

#[derive(Debug, Clone)]
pub enum Extension {
    Jpeg,
    Png,
    Webp,
}

#[derive(Debug, Clone)]
pub struct ImgData {
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
    let lowercase_path = Path::new(path).extension().and_then(|s| s.to_str()).unwrap().to_string();
    match lowercase_path.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => Ok(Extension::Jpeg),
        "png" => Ok(Extension::Png),
        "webp" => Ok(Extension::Webp),
        _ => {
            if path.ends_with("jpg") || path.ends_with("jpeg") {
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
        Ok(Extension::Jpeg) => {
            let jpeg = jpeg::JpegImage::open(&path).map_err(|e| e.to_string())?;
            Ok(Img {
                extension: Extension::Jpeg,
                data: ImgData {
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
                    jpeg: None,
                    png: None,
                    webp: Some(webp),
                },
            })
        },
        Err(e) => Err(e),
    }
}

pub fn compress(data: &mut ImgData, extension: &Extension) -> Result<(), String> {
    match extension {
        Extension::Jpeg => {
            match &mut data.jpeg {
                Some(jpeg) => {
                    jpeg.compress()
                },
                None => return Err("Failed to save jpeg image".to_string()),
            }
        },
        Extension::Png => {
            match &mut data.png {
                Some(png) => {
                    png.compress()
                },
                None => return Err("Failed to save png image".to_string()),
            }
        },
        Extension::Webp => {
            match &mut data.webp {
                Some(webp) => {
                    webp.compress()
                },
                None => return Err("Failed to save webp image".to_string()),
            }
        },
    }
}

pub fn convert(source_img: &mut Img, destination_extension: &Extension) -> Result<Img, String> {
    match source_img.extension {
        Extension::Jpeg => {
            match &source_img.data.jpeg {
                Some(jpeg) => {
                    let dynamic_image = jpeg.image.clone();
                    match destination_extension {
                        Extension::Jpeg => {
                            Ok(Img {
                                extension: Extension::Jpeg,
                                data: ImgData {
                                    jpeg: Some(jpeg.clone()),
                                    png: None,
                                    webp: None,
                                },
                            })
                        },
                        Extension::Png => {
                            Ok(source_img.clone())
                        },
                        Extension::Webp => {
                            let webp = webp::WebpImage::import(dynamic_image, jpeg.filepath_input.clone(), jpeg.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Webp,
                                data: ImgData {
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
                        Extension::Jpeg => {
                            let jpeg = jpeg::JpegImage::import(dynamic_image, png.filepath_input.clone(), png.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Jpeg,
                                data: ImgData {
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
                        Extension::Jpeg => {
                            let jpeg = jpeg::JpegImage::import(dynamic_image, webp.filepath_input.clone(), webp.metadata_input.clone())?;
                            Ok(Img {
                                extension: Extension::Jpeg,
                                data: ImgData {
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
    println!("{} -> {}", before_path, after_path);
    println!("{} -> {} ({:.1}%)", before_size, after_size, (after_size as f64 / before_size as f64) * 100.0);
}

pub fn save_image(path: Option<&String>, data: &mut ImgData, extension: &Extension, quality: Option<f32>) -> Result<String, String> {
    match extension {
        Extension::Jpeg => {
            match data.jpeg {
                Some(ref mut jpeg) => {
                    jpeg.save(path, quality)?;
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
                    png.save(path, quality)?;
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
                    webp.save(path, quality)?;
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
