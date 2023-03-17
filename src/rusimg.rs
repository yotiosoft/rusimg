pub mod jpeg;
pub mod png;

use image::DynamicImage;

pub trait Rusimg {
    fn new(&mut self, image: DynamicImage) -> Result<Self, String> where Self: Sized;
    fn open(path: &str) -> Result<Self, String> where Self: Sized;
    fn save(&mut self, path: &Option<String>) -> Result<(), String>;
    fn compress(&mut self) -> Result<(), String>;
}
