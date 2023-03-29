mod bmp;
mod jpeg;
mod png;
mod webp;

use std::path::Path;
use std::fs::Metadata;
use std::io::{Read, Write};
use image::DynamicImage;

pub trait Rusimg {
    fn import(image: DynamicImage, source_path: String, source_metadata: Metadata, new_extension: Extension) -> Result<Self, String> where Self: Sized;
    fn open(path: &str) -> Result<Self, String> where Self: Sized;
    fn save(&mut self, path: Option<&String>) -> Result<(), String>;
    fn compress(&mut self, quality: Option<f32>) -> Result<(), String>;
    fn resize(&mut self, resize_ratio: u8) -> Result<(), String>;
    fn trim(&mut self, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<(), String>;
    fn grayscale(&mut self);
    fn view(&self) -> Result<(), String>;

    fn save_filepath(source_filepath: &String, destination_filepath: Option<&String>, new_extension: &String) -> String {
        if let Some(path) = destination_filepath {
            if Path::new(path).is_dir() {
                let filename = Path::new(&source_filepath).file_name().expect("Failed to get filename").to_str().expect("Failed to convert filename to string");
                Path::new(path).join(filename).with_extension(new_extension).to_str().expect("Failed to convert path to string").to_string()
            }
            else {
                path.to_string()
            }
        }
        else {
            Path::new(&source_filepath).with_extension(new_extension).to_str().expect("Failed to convert path to string").to_string()
        }
    }
}

pub struct ImgStruct {
    pub extension: Extension,
    pub image: DynamicImage,
    image_bytes: Option<Vec<u8>>,
    width: usize,
    height: usize,
    operations_count: u32,
    extension_str: String,
    pub metadata_input: Metadata,
    pub metadata_output: Option<Metadata>,
    pub filepath_input: String,
    pub filepath_output: Option<String>,
}

impl Rusimg for ImgStruct {
    fn import(image: DynamicImage, source_path: String, source_metadata: Metadata, new_extension: Extension) -> Result<Self, String> {
        let (width, height) = (image.width() as usize, image.height() as usize);

        Ok(Self {
            extension: new_extension,
            image,
            image_bytes: None,
            width,
            height,
            operations_count: 0,
            extension_str: "jpg".to_string(),
            metadata_input: source_metadata,
            metadata_output: None,
            filepath_input: source_path,
            filepath_output: None,
        })
    }

    fn open(path: &str) -> Result<Self, String> {
        let mut raw_data = std::fs::File::open(path).map_err(|_| "Failed to open file".to_string())?;
        let mut buf = Vec::new();
        raw_data.read_to_end(&mut buf).map_err(|_| "Failed to read file".to_string())?;
        let metadata_input = raw_data.metadata().map_err(|_| "Failed to get metadata".to_string())?;

        let image = image::load_from_memory(&buf).map_err(|_| "Failed to open image".to_string())?;
        let (width, height) = (image.width() as usize, image.height() as usize);

        let extension_str = Path::new(path).extension().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let extension = match extension_str.as_str() {
            "bmp" => Extension::Bmp,
            "jpg" | "jpeg" => Extension::Jpeg,
            "png" => Extension::Png,
            "webp" => Extension::Webp,
            _ => return Err("Unsupported file extension".to_string()),
        };

        Ok(Self {
            extension,
            image,
            image_bytes: None,
            width,
            height,
            operations_count: 0,
            extension_str,
            metadata_input,
            metadata_output: None,
            filepath_input: path.to_string(),
            filepath_output: None,
        })
    }

    fn save(&mut self, path: Option<&String>) -> Result<(), String> {
        let save_path = Self::save_filepath(&self.filepath_input, path, &self.extension_str);
        
        // image_bytes == None の場合、DynamicImage を 保存
        if self.image_bytes.is_none() {
            self.image.save(&save_path).map_err(|e| format!("Failed to save image: {}", e.to_string()))?;
            self.metadata_output = Some(std::fs::metadata(&save_path).map_err(|_| "Failed to get metadata".to_string())?);
        }
        // image_bytes != None の場合、mozjpeg::Compress で圧縮したバイナリデータを保存
        else {
            let mut file = std::fs::File::create(&save_path).map_err(|_| "Failed to create file".to_string())?;
            file.write_all(&self.image_bytes.as_ref().unwrap()).map_err(|_| "Failed to write file".to_string())?;
            self.metadata_output = Some(file.metadata().map_err(|_| "Failed to get metadata".to_string())?);
        }

        self.filepath_output = Some(save_path);

        Ok(())
    }

    fn compress(&mut self, quality: Option<f32>) -> Result<(), String> {
        match self.extension {
            Extension::Bmp => {
                return Err("BMP format does not support compression".to_string());
            },
            Extension::Jpeg => {
                jpeg::compress(&mut self.image, quality, &mut self.image_bytes);
            },
            Extension::Png => {
                png::compress(&mut self.image, quality, &mut self.image_bytes);
            },
            Extension::Webp => {
                webp::compress(&mut self.image, quality, &mut self.image_bytes);
            },
        }
    }

    fn resize(&mut self, resize_ratio: u8) -> Result<(), String> {
        let nwidth = (self.width as f32 * (resize_ratio as f32 / 100.0)) as usize;
        let nheight = (self.height as f32 * (resize_ratio as f32 / 100.0)) as usize;
        
        self.image = self.image.resize(nwidth as u32, nheight as u32, image::imageops::FilterType::Lanczos3);

        println!("Resize: {}x{} -> {}x{}", self.width, self.height, nwidth, nheight);

        self.width = nwidth;
        self.height = nheight;

        self.operations_count += 1;
        Ok(())
    }

    fn trim(&mut self, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<(), String> {
        let mut w = trim_wh.0;
        let mut h = trim_wh.1;
        if self.width < (trim_xy.0 + w) as usize || self.height < (trim_xy.1 + h) as usize {
            if self.width > trim_xy.0 as usize && self.height > trim_xy.1 as usize {
                w = if self.width < (trim_xy.0 + w) as usize { self.width as u32 - trim_xy.0 } else { trim_wh.0 };
                h = if self.height < (trim_xy.1 + h) as usize { self.height as u32 - trim_xy.1 } else { trim_wh.1 };
                println!("Required width or height is larger than image size. Corrected size: {}x{} -> {}x{}", trim_wh.0, trim_wh.1, w, h);
            }
            else {
                return Err(format!("Trim: Invalid trim point: {}x{}", trim_xy.0, trim_xy.1));
            }
        }

        self.image = self.image.crop(trim_xy.0, trim_xy.1, w, h);

        println!("Trim: {}x{} -> {}x{}", self.width, self.height, w, h);

        self.width = w as usize;
        self.height = h as usize;

        self.operations_count += 1;
        Ok(())
    }

    fn grayscale(&mut self) {
        self.image = self.image.grayscale();
        println!("Grayscale: Done.");
        self.operations_count += 1;
    }

    fn view(&self) -> Result<(), String> {
        let conf_width = self.width as f64 / std::cmp::max(self.width, self.height) as f64 * 100 as f64;
        let conf_height = self.height as f64 / std::cmp::max(self.width, self.height) as f64 as f64 * 50 as f64;
        let conf = viuer::Config {
            absolute_offset: false,
            width: Some(conf_width as u32),
            height: Some(conf_height as u32),    
            ..Default::default()
        };

        viuer::print(&self.image, &conf).map_err(|e| format!("Failed to view image: {}", e.to_string()))?;

        Ok(())
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

pub fn trim(image: &mut Img, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<(), String> {
    match image.extension {
        Extension::Bmp => {
            match &mut image.data.bmp {
                Some(bmp) => {
                    bmp.trim(trim_xy, trim_wh)
                },
                None => return Err("Failed to save bmp image".to_string()),
            }
        },
        Extension::Jpeg => {
            match &mut image.data.jpeg {
                Some(jpeg) => {
                    jpeg.trim(trim_xy, trim_wh)
                },
                None => return Err("Failed to save jpeg image".to_string()),
            }
        },
        Extension::Png => {
            match &mut image.data.png {
                Some(png) => {
                    png.trim(trim_xy, trim_wh)
                },
                None => return Err("Failed to save png image".to_string()),
            }
        },
        Extension::Webp => {
            match &mut image.data.webp {
                Some(webp) => {
                    webp.trim(trim_xy, trim_wh)
                },
                None => return Err("Failed to save webp image".to_string()),
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
