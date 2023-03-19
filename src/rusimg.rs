mod jpeg;
mod png;
mod webp;

use std::path::Path;
use image::DynamicImage;
use std::fs::Metadata;

pub trait Rusimg {
    fn new(image: DynamicImage, source_path: String, source_metadata: Metadata) -> Result<Self, String> where Self: Sized;
    fn open(path: &str) -> Result<Self, String> where Self: Sized;
    fn save(&mut self, path: &Option<String>) -> Result<(), String>;
    fn compress(&mut self) -> Result<(), String>;
}

pub enum Extension {
    Jpeg,
    Png,
    Webp,
}

pub struct ImgData {
    jpeg: Option<jpeg::JpegImage>,
    png: Option<png::PngImage>,
    webp: Option<webp::WebpImage>,
}

pub struct Img {
    pub extension: Extension,
    pub data: ImgData,
}

pub fn get_extension(path: &str) -> Result<Extension, String> {
    match Path::new(path).extension().and_then(|s| s.to_str()) {
        Some("jpg") | Some("jpeg") => Ok(Extension::Jpeg),
        Some("png") => Ok(Extension::Png),
        Some("webp") => Ok(Extension::Webp),
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

pub fn convert(data: &mut ImgData, source_extension: &Extension, destination_extension: &Extension) -> Result<Img, String> {
    match source_extension {
        Extension::Jpeg => {
            match &mut data.jpeg {
                Some(jpeg) => {
                    let dynamic_image = jpeg.image.clone();
                    match destination_extension {
                        Extension::Jpeg => {
                            Err("Source and destination extensions are the same".to_string())
                        },
                        Extension::Png => {
                            let png = png::PngImage::new(dynamic_image, jpeg.filepath_input.clone(), jpeg.metadata_input.clone())?;
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
                            let webp = webp::WebpImage::new(dynamic_image, jpeg.filepath_input.clone(), jpeg.metadata_input.clone())?;
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
            match &mut data.png {
                Some(png) => {
                    let dynamic_image = png.image.clone();
                    match destination_extension {
                        Extension::Jpeg => {
                            let jpeg = jpeg::JpegImage::new(dynamic_image, png.filepath_input.clone(), png.metadata_input.clone())?;
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
                            Err("Source and destination extensions are the same".to_string())
                        },
                        Extension::Webp => {
                            let webp = webp::WebpImage::new(dynamic_image, png.filepath_input.clone(), png.metadata_input.clone())?;
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
            match &mut data.webp {
                Some(webp) => {
                    let dynamic_image = webp.image.clone();
                    match destination_extension {
                        Extension::Jpeg => {
                            let jpeg = jpeg::JpegImage::new(dynamic_image, webp.filepath_input.clone(), webp.metadata_input.clone())?;
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
                            let png = png::PngImage::new(dynamic_image, webp.filepath_input.clone(), webp.metadata_input.clone())?;
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
                            Err("Source and destination extensions are the same".to_string())
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

pub fn save_image(path: &Option<String>, data: &mut ImgData, extension: &Extension) -> Result<(), String> {
    match extension {
        Extension::Jpeg => {
            match data.jpeg {
                Some(ref mut jpeg) => {
                    jpeg.save(path)?;
                    save_print(
                        &jpeg.filepath_input, &jpeg.filepath_output.as_ref().unwrap(), 
                        jpeg.metadata_input.len(), jpeg.metadata_output.as_ref().unwrap().len()
                    );
                    Ok(())
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
                    Ok(())
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
                    Ok(())
                },
                None => return Err("Failed to save webp image".to_string()),
            }
        },
    }
}
