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

struct Img {
    extension: Extension,
    jpeg: Option<jpeg::JpegImage>,
    png: Option<png::PngImage>,
}

fn get_extension(path: &str) -> Result<compress::Extension, String> {
    match Path::new(path).extension().and_then(|s| s.to_str()) {
        Some("jpg") | Some("jpeg") => Ok(compress::Extension::Jpeg),
        Some("png") => Ok(compress::Extension::Png),
        _ => Err("Unsupported file extension".to_string()),
    }
}

fn get_extension_string(extension: &compress::Extension) -> String {
    match extension {
        compress::Extension::Jpeg => "jpg".to_string(),
        compress::Extension::Png => "png".to_string(),
    }
}

fn main() -> Result<(), String> {
    let args = parse::parser();
    let image = open_image(&args.souce_path)?;

    // 圧縮
    let compressed_img = compress::compress(image.data, image.width, image.height, &image.extension).map_err(|e| e.to_string())?;

    // 出力
    let output_path = match args.destination_path {
        Some(path) => path,
        None => "output".to_string() + "." + &get_extension_string(&image.extension),
    };
    output_image(&output_path, Img {
        width: image.width,
        height: image.height,
        data: compressed_img,
        extension: image.extension,
    })?;

    Ok(())
}
