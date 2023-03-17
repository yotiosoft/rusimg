use std::path::Path;

mod parse;
mod rusimg;

use rusimg::jpeg;
use rusimg::png;
use rusimg::Rusimg;

pub enum Extension {
    Jpeg,
    Png,
}

struct ImgData {
    jpeg: Option<jpeg::JpegImage>,
    png: Option<png::PngImage>,
}

struct Img {
    extension: Extension,
    data: ImgData,
}

fn get_extension(path: &str) -> Result<Extension, String> {
    match Path::new(path).extension().and_then(|s| s.to_str()) {
        Some("jpg") | Some("jpeg") => Ok(Extension::Jpeg),
        Some("png") => Ok(Extension::Png),
        _ => Err("Unsupported file extension".to_string()),
    }
}

fn open_image(path: &str) -> Result<Img, String> {
    match get_extension(&path) {
        Ok(Extension::Jpeg) => {
            let jpeg = jpeg::JpegImage::open(&path).map_err(|e| e.to_string())?;
            Ok(Img {
                extension: Extension::Jpeg,
                data: ImgData {
                    jpeg: Some(jpeg),
                    png: None,
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
                },
            })
        },
        Err(e) => Err(e),
    }
}

fn compress(data: &mut ImgData, extension: &Extension) -> Result<(), String> {
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
    }
}

fn save_print(before_path: &String, after_path: &String, before_size: u64, after_size: u64) {
    println!("{} -> {}", before_path, after_path);
    println!("{} -> {}", before_size, after_size);
    println!("{}%", (after_size as f64 / before_size as f64) * 100.0);
}

fn save_image(path: &Option<String>, data: &mut ImgData, extension: &Extension) -> Result<(), String> {
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
    }
}

fn main() -> Result<(), String> {
    let args = parse::parser();
    let mut image = open_image(&args.souce_path)?;

    // 圧縮
    match compress(&mut image.data, &image.extension) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    // 出力
    let output_path = match args.destination_path {
        Some(path) => Some(path),
        None => None,
    };
    save_image(&output_path, &mut image.data, &image.extension)?;

    Ok(())
}
