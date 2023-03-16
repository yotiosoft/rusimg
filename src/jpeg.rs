extern crate mozjpeg;
use mozjpeg::{Compress, ColorSpace, ScanMode};

struct JpegImage {
    pub image: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl JpegImage {
    pub fn new(image: Vec<u8>, width: usize, height: usize) -> Self {
        Self {
            image,
            width,
            height,
        }
    }

    pub fn compress(&self) -> Result<Vec<u8>, String> {
        let mut compress = Compress::new(ColorSpace::JCS_RGB);
        compress.set_scan_optimization_mode(ScanMode::AllComponentsTogether);
        compress.set_size(self.width, self.height);
        compress.set_mem_dest();
        compress.start_compress();
        compress.write_scanlines(&self.image);
        compress.finish_compress();

        compress.data_to_vec().map_err(|_| "Failed to compress jpeg image".to_string())
    }
}
