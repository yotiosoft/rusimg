use image;
use std::io::{BufWriter, Write};
use std::path::Path;

mod parse;
mod compress;

struct Img {
    width: usize,
    height: usize,
    data: Vec<u8>,
    extension: compress::Extension,
}

fn open_image(path: &str) -> Result<Img, String> {
    let image = image::open(path).map_err(|_| "Failed to open image".to_string())?;
    let (width, height) = (image.width() as usize, image.height() as usize);
    
    let externsion = get_extension(path)?;

    Ok(Img {
        width: width as usize,
        height: height as usize,
        data: image.to_rgb8().into_raw(),
        extension: externsion,
    })
}

fn output_image(path: &str, image: Img) -> Result<(), String> {
    let mut file = BufWriter::new(std::fs::File::create(path).map_err(|_| "Failed to create file".to_string())?);
    file.write_all(&image.data).map_err(|_| "Failed to write file".to_string())?;

    Ok(())
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
