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

fn save_image(path: &str, data: &mut ImgData, extension: &Extension) -> Result<(), String> {
    match extension {
        Extension::Jpeg => {
            match data.jpeg {
                Some(ref jpeg) => {
                    jpeg.save(&path)
                },
                None => return Err("Failed to save jpeg image".to_string()),
            }
        },
        Extension::Png => {
            match data.png {
                Some(ref png) => {
                    png.save(&path)
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
        Some(path) => path,
        None => "output".to_string() + "." + &get_extension_string(&image.extension),
    };
    //save_image(&output_path, &mut image.data, &image.extension)?;

    Ok(())
}
