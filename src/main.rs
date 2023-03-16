use image;

mod parse;
mod compress;

fn open_image(path: &str) -> Result<Vec<u8>, String> {
    let image = image::open(path).map_err(|_| "Failed to open image".to_string())?;
    let image = image.to_rgb8();
    let image = image.into_raw();
    Ok(image)
}

fn main() -> Result<(), String> {
    let args = parse::parser();
    let image = open_image(&args.souce_path)?;

    Ok(())
}
