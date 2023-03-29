use mozjpeg::{Compress, ColorSpace, ScanMode};
use image::DynamicImage;

use std::fs::Metadata;
use std::io::{Read, Write};
use std::path::Path;

pub fn compress(mut image: &DynamicImage, quality: Option<f32>) -> Result<Vec<u8>, String> {
    let quality = quality.unwrap_or(75.0);  // default quality: 75.0

    let image_bytes = image.clone().into_bytes();

    let (width, height) = (image.width() as usize, image.height() as usize);

    let mut compress = Compress::new(ColorSpace::JCS_RGB);
    compress.set_scan_optimization_mode(ScanMode::AllComponentsTogether);
    compress.set_size(width, height);
    compress.set_mem_dest();
    compress.set_quality(quality);
    compress.start_compress();
    compress.write_scanlines(&image_bytes);
    compress.finish_compress();

    println!("Compress: Done.");

    compress.data_to_vec().map_err(|_| "Failed to compress image".to_string())
}
