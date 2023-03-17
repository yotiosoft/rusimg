pub mod jpeg;
pub mod png;

pub trait Rusimg {
    fn open(path: &str) -> Result<Self, String> where Self: Sized;
    fn save(&mut self, path: &Option<String>) -> Result<(), String>;
    fn compress(&mut self) -> Result<(), String>;
}
