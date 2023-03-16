use image;
use std::io::{BufWriter, Write};

mod parse;
mod compress;

struct Img {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

fn open_image(path: &str) -> Result<Img, String> {
    let image = image::open(path).map_err(|_| "Failed to open image".to_string())?;
    let (width, height) = (image.width() as usize, image.height() as usize);

    Ok(Img {
        width: width as usize,
        height: height as usize,
        data: image.to_rgb8().into_raw(),
    })
}

fn output_image(path: &str, image: Img) -> Result<(), String> {
    let mut file = BufWriter::new(std::fs::File::create(path).map_err(|_| "Failed to create file".to_string())?);
    file.write_all(&image.data).map_err(|_| "Failed to write file".to_string())?;

    Ok(())
}

fn main() -> Result<(), String> {
    let args = parse::parser();
    let image = open_image(&args.souce_path)?;

    // 圧縮
    let compressed_img = compress::compress(image.data, image.width, image.height, compress::Extension::Jpeg).map_err(|e| e.to_string())?;

    // 出力
    let output_path = match args.destination_path {
        Some(path) => path,
        None => "output.jpg".to_string(),
    };
    output_image(&output_path, Img {
        width: image.width,
        height: image.height,
        data: compressed_img,
    })?;

    Ok(())
}
