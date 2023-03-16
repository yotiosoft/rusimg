mod jpeg;

pub enum Extension {
    Jpeg,
    Png,
}

pub fn compress(image: Vec<u8>, width: usize, height: usize, extension: Extension) -> Result<Vec<u8>, String> {
    match extension {
        Extension::Jpeg => jpeg::compress(image, width, height),
        Extension::Png => Err("PNG compression not implemented".to_string()),
    }
}
