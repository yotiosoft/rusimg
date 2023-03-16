extern crate oxipng;

pub fn compress(image: Vec<u8>) -> Result<Vec<u8>, String> {
    println!("compressing png image...");
    match oxipng::optimize_from_memory(&image as &[u8], &oxipng::Options::default()) {
        Ok(data) => Ok(data),
        Err(e) => match e {
            oxipng::PngError::DeflatedDataTooLong(s) => Err(format!("deflated data too long: {}", s)),
            oxipng::PngError::TimedOut => Err("timed out".to_string()),
            oxipng::PngError::NotPNG => Err("not png".to_string()),
            oxipng::PngError::APNGNotSupported => Err("apng not supported".to_string()),
            oxipng::PngError::InvalidData => Err("invalid data".to_string()),
            oxipng::PngError::TruncatedData => Err("truncated data".to_string()),
            oxipng::PngError::ChunkMissing(s) => Err(format!("chunk missing: {}", s)),
            oxipng::PngError::Other(s) => Err(format!("other: {}", s)),
            _ => Err("unknown error".to_string()),
        }
    }
}
