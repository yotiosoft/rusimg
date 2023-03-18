pub mod jpeg;
pub mod png;
pub mod webp;

use image::DynamicImage;
use std::fs::Metadata;

pub trait Rusimg {
    fn new(image: DynamicImage, source_path: String, source_metadata: Metadata) -> Result<Self, String> where Self: Sized;
    fn open(path: &str) -> Result<Self, String> where Self: Sized;
    fn save(&mut self, path: &Option<String>) -> Result<(), String>;
    fn compress(&mut self) -> Result<(), String>;
}
