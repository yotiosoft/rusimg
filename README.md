# rusimg

A tool to run DeepL translations on command line written by Rust.

## Install

```bash
$ cargo install rusimg
```

## Binary crate
### Options

|option|description|
|--|--|
|-o, --output|Specify output directory or output file name.|
|-c, --convert|Image Conversion.（jpeg, png, webp, bmp）|
|-r, --resize|Image resizing. (specified by scaling factor: (0, 100])|
|-t, --trim|Image cropping.|

### Image Conversion: options
|format|option|
|--|--|
|jpeg|-c jpeg|
|png|-c png|
|webp|-c webp|
|bmp|-c bmp|

## Library crate

### open_image()
Given a file path, open_image() returns struct RusImg, which contains the data for that image.
Struct RusImg has public processing functions for that image in ``RusimgTrait``.
```rust
/// Open an image file.
pub fn open_image(path: &str) -> Result<RusImg, RusimgError> {
    let path_buf = PathBuf::from(path);
    let img = rusimg::open_image(&path_buf)?;
    Ok(img)
}
```

### struct RusImg
struct RusImg holds the file extension Extension and the image data RusimgTrait.
```rust
pub struct RusImg {
    pub extension: Extension,
    pub data: Box<(dyn RusimgTrait)>,
}
```

### Extension
enum Extension indicates the file extension.  
ExternalFormat(String) is provided for the library crate users to use if they wish to implement their own alternate image file format.
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Extension {
    Bmp,
    Jpeg,
    Png,
    Webp,
    ExternalFormat(String),
}
impl fmt::Display for Extension {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Extension::Bmp => write!(f, "bmp"),
            Extension::Jpeg => write!(f, "jpeg"),
            Extension::Png => write!(f, "png"),
            Extension::Webp => write!(f, "webp"),
            Extension::ExternalFormat(s) => write!(f, "{}", s),
        }
    }
}
```

### RusimgTrait
Trait that implements the processing functions for each file format.  
If you want to implement your own separate image file formats, you must implement these functions.
```rust
pub trait RusimgTrait {
    fn import(image: DynamicImage, source_path: PathBuf, source_metadata: Metadata) -> Result<Self, RusimgError> where Self: Sized;
    fn open(path: PathBuf, image_buf: Vec<u8>, metadata: Metadata) -> Result<Self, RusimgError> where Self: Sized;
    fn save(&mut self, path: Option<PathBuf>) -> Result<(), RusimgError>;
    fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError>;
    fn resize(&mut self, resize_ratio: u8) -> Result<ImgSize, RusimgError>;
    fn trim(&mut self, trim_xy: (u32, u32), trim_wh: (u32, u32)) -> Result<ImgSize, RusimgError>;
    fn grayscale(&mut self);

    fn set_dynamic_image(&mut self, image: DynamicImage) -> Result<(), RusimgError>;

    fn get_dynamic_image(&mut self) -> Result<DynamicImage, RusimgError>;
    fn get_source_filepath(&self) -> PathBuf;
    fn get_destination_filepath(&self) -> Option<PathBuf>;
    fn get_metadata_src(&self) -> Metadata;
    fn get_metadata_dest(&self) -> Option<Metadata>;
    fn get_size(&self) -> ImgSize;

    fn save_filepath(&self, source_filepath: &PathBuf, destination_filepath: Option<PathBuf>, new_extension: &String) -> Result<PathBuf, RusimgError> {
        if let Some(path) = destination_filepath {
            if Path::new(&path).is_dir() {
                let filename = match Path::new(&source_filepath).file_name() {
                    Some(filename) => filename,
                    None => return Err(RusimgError::FailedToGetFilename(source_filepath.clone())),
                };
                Ok(Path::new(&path).join(filename).with_extension(new_extension))
            }
            else {
                Ok(path)
            }
        }
        else {
            Ok(Path::new(&source_filepath).with_extension(new_extension))
        }
    }
}
```
