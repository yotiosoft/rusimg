use std::fs::File;
use std::io::{BufWriter, Write, Read};
use std::path::Path;

mod parse;
mod jpeg;
mod png;

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

fn get_extension_string(extension: &Extension) -> String {
    match extension {
        Extension::Jpeg => "jpg".to_string(),
        Extension::Png => "png".to_string(),
    }
}

fn open_image(path: &str) -> Result<ImgData, String> {
    match get_extension(&path) {
        Ok(Extension::Jpeg) => {
            let jpeg = jpeg::JpegImage::open(&path).map_err(|e| e.to_string())?;
            Ok(ImgData {
                jpeg: Some(jpeg),
                png: None,
            })
        },
        Ok(Extension::Png) => {
            let png = png::PngImage::open(&path).map_err(|e| e.to_string())?;
            Ok(ImgData {
                jpeg: None,
                png: Some(png),
            })
        },
        Err(e) => Err(e),
    }
}

fn compress(data: ImgData, extension: &Extension) -> Result<(), String> {
    match extension {
        Extension::Jpeg => {
            let jpeg = data.jpeg.unwrap();
            jpeg.compress()
        },
        Extension::Png => {
            let png = data.png.unwrap();
            png.compress()
        },
    }
}

fn save_image(path: &str, data: ImgData, extension: &Extension) -> Result<(), String> {
    match extension {
        Extension::Jpeg => {
            let jpeg = data.jpeg.unwrap();
            jpeg.save(&path)
        },
        Extension::Png => {
            let png = data.png.unwrap();
            png.save(&path)
        },
    }
}

fn main() -> Result<(), String> {
    let args = parse::parser();
    let image = open_image(&args.souce_path)?;

    // 圧縮
    compress(image.data);

    // 出力
    let output_path = match args.destination_path {
        Some(path) => path,
        None => "output".to_string() + "." + &get_extension_string(&image.extension),
    };
    save_image(&output_path, image)?;

    Ok(())
}
