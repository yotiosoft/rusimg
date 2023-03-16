extern crate mozjpeg;
use mozjpeg::{Compress, ColorSpace, ScanMode};

pub fn compress(image: Vec<u8>, width: usize, height: usize) -> Result<Vec<u8>, String> {
    let mut compress = Compress::new(ColorSpace::JCS_RGB);
    compress.set_scan_optimization_mode(ScanMode::AllComponentsTogether);
    compress.set_size(width, height);
    compress.set_mem_dest();
    compress.start_compress();
    compress.write_scanlines(&image);
    compress.finish_compress();

    compress.data_to_vec().map_err(|_| "Failed to compress jpeg image".to_string())
}
