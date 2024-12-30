# rusimg

A tool to run DeepL translations on command line written by Rust.

## Features

- Image Conversion (jpeg, png, webp, bmp)
- Set Conversion Quality
- Image Resizing
- Image Cropping
- Grayscale Conversion
- Save the image

### Image Conversion

Rusimg can convert images to the following formats.  

- For binary crates, the conversion format can be specified with the ``-c`` option.
- For library crates, the conversion format can be specified by calling the ``rusimg::RusImg.convert()`` function.

| format | binary crate option | library crate extension |
| ------ | ------------------- | ----------------------- |
| jpeg   | -c jpeg             | Extension::Jpeg         |
| png    | -c png              | Extension::Png          |
| webp   | -c webp             | Extension::Webp         |
| bmp    | -c bmp              | Extension::Bmp          |


### Set Conversion Quality

Rusimg can set the quality of the converted image. This depends on each image format.

- For binary crates, the quality can be specified with the ``-q`` option. 
- For library crates, the quality can be specified by calling the ``rusimg::RusImg.compress()`` function.

| format | quality                                                      | note                                                         |
| ------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
| jpeg   | 0-100                                                        | By default, the quality is set to 75.                        |
| png    | [0, 17.0], (17.0, 34.0], (34.0, 51.0], (51.0, 68.0], (68.0, 85.0], (85.0, 100.0] | Because the ``oxipng`` crate must be set to the 6 compression levels, input values will be converted into 6 levels. By default, the quality is set to 68.0-85.0. |
| webp   | 0-100                                                        | By default, the quality is set to 75.0.                      |
| bmp    | none                                                         | BMP does not have a quality setting because it is a lossless format. |

### Image Resizing

Resize images. The resize ratio is specified by a scaling factor (0, 100].

- For binary crates, the resize ratio can be specified with the ``-r`` option.
- For library crates, the resize ratio can be specified by calling the ``rusimg::RusImg.resize()`` function.

### Image Cropping

Crop images.

- For binary crates, the crop size can be specified with the ``-t`` option.
- For library crates, the crop size can be specified by calling the ``rusimg::RusImg.trim()`` or ``rusimg::RusImg.trim_rect()`` function.

### Grayscale Conversion

Convert images to grayscale.

- For binary crates, the grayscale conversion can be specified with the ``-g`` option.
- For library crates, the grayscale conversion can be specified by calling the ``rusimg::RusImg.grayscale()`` function.

### Save the image

Save the image to the specified file path.

## Binary crate

### Install

Use ``cargo`` to install the binary crate.

```bash
$ cargo install rusimg
```

The binary crates contain ``app`` features required to run the application by default, but this is not necessary when used as a library.

### Binary crate options

|option|description|
|--|--|
|-o, --output \<OUTPUT\>|Specify output directory or output file name. \<OUTPUT\> is the output directory or output file name.|
|-c, --convert \<CONVERT\>|Image Conversion（jpeg, png, webp, bmp）. \<CONVERT\> is the image format to convert to.|
|-r, --resize \<RESIZE\>|Image resizing (specified by scaling factor: (0, 100]). \<RESIZE\> is the scaling factor percentage.|
|-t, --trim \<TRIM\>|Image cropping. Input format: 'XxY+W+H' (e.g.100x100+50x50)|
|-g, --grayscale|Grayscale conversion.|
|-q, --quality \<QUALITY\>|Image quality. \<QUALITY\> is the image quality (0, 100].|
|-a, --append \<APPEND\>|Append a string to the file name. \<APPEND\> is the string to append. (e.g. -a "_new")|
|-d, --double-extension|Set output file path to double extension. (e.g. "image.jpg.webp")|
|-D, --delete|Delete the original file.|
|-y, --yes|If the destination file already exists, overwrite it without asking.|
|-n, --no|If the destination file already exists, do not overwrite it without asking.|
|-T, --threads \<THREADS\>|Number of threads to use. \<THREADS\> is the number of threads to use. Default: 4|
|-v, --view|View the image. Use ``viuer`` crate.|
|-h, --help|Display help message.|
|-V, --version|Display version information.|
|--recursive|Recursively process all files in the directory.|

## Library crate

### Install

Use ``cargo`` to add the library crate.

```bash
$ cargo add rusimg --no-default-features --features bmp,jpeg,png,webp
```

Or, add this to your ``Cargo.toml``.

```toml
[dependencies]
rusimg = { version = "0.1.0", default-features = false, features = ["bmp", "jpeg", "png", "webp"] }
```

Note that this crate includes the ``app`` feature by default, **which is only necessary for the binary crate but not for the library crate**.  
This feature includes following dependencies: ``clap``, ``regex``, ``viuer``, ``glob``, ``colored``, ``tokio``, ``futures``.

If you don't use the specified image format, you can remove it from the features.  
For example, if don't use the bmp format, leave ``bmp`` out of the features.

```toml
[dependencies]
rusimg = { version = "0.1.0", default-features = false, features = ["jpeg", "png", "webp"] }
```

### Library crate typical features

#### rusimg::open_image()
Given a file path, open_image() returns struct RusImg, which contains the data for that image.
Struct ``RusImg`` has public processing functions for that image in ``RusimgTrait``.

```rust
pub fn open_image(path: &Path) -> Result<RusImg, RusimgError>;
```

#### rusimg::RusImg.convert()

Converts the image to the specified format.  
If conversion is successful, the image data is updated in the struct RusImg.

```rust
pub fn convert(&mut self, new_extension: &Extension) -> Result<(), RusimgError>;
```

#### rusimg::RusImg.save_image()

``save_image()`` saves the image to the specified file path.  
If the destination file path is not specified, the image is saved to the same file path as the source file (excluding the file extension).

```rust
pub fn save_image(&mut self, path: Option<&str>) -> Result<SaveStatus, RusimgError>;
```

### Structs

#### struct RusImg
struct ``RusImg`` holds the file extension and the image data (``RusimgTrait``).  
``RusimgTrait`` is a trait that contains the image processing functions, but struct ``RusImg`` implements these wrapper functions.
```rust
pub struct RusImg {
    pub extension: Extension,
    pub data: Box<(dyn RusimgTrait)>,
}
```

##### struct RusImg implements

struct ``RusImg`` implements following functions.

```rust
impl RusImg {
    /// Get image size.
    pub fn get_image_size(&self) -> Result<ImgSize, RusimgError>;

    /// Resize an image.
    /// It must be called after open_image().
    /// Set ratio to 100 to keep the original size.
    pub fn resize(&mut self, ratio: u8) -> Result<ImgSize, RusimgError>;

    /// Trim an image. Set the trim area with four u32 values: x, y, w, h.
    /// It must be called after open_image().
    pub fn trim(&mut self, trim_x: u32, trim_y: u32, trim_w: u32, trim_h: u32) -> Result<ImgSize, RusimgError>;
    /// Trim an image. Set the trim area with a rusimg::Rect object.
    /// It must be called after open_image().
    pub fn trim_rect(&mut self, trim_area: Rect) -> Result<ImgSize, RusimgError>;

    /// Grayscale an image.
    /// It must be called after open_image().
    pub fn grayscale(&mut self) -> Result<(), RusimgError>;

    /// Compress an image.
    /// It must be called after open_image().
    /// Set quality to 100 to keep the original quality.
    pub fn compress(&mut self, quality: Option<f32>) -> Result<(), RusimgError>;

    /// Convert an image to another format.
    /// And replace the original image with the new one.
    /// It must be called after open_image().
    pub fn convert(&mut self, new_extension: &Extension) -> Result<(), RusimgError>;

    /// Set a DynamicImage to an Img.
    pub fn set_dynamic_image(&mut self, image: DynamicImage) -> Result<(), RusimgError>;

    /// Get a DynamicImage from an Img.
    pub fn get_dynamic_image(&mut self) -> Result<DynamicImage, RusimgError>;

    /// Get file extension.
    pub fn get_extension(&self) -> Extension;

    /// Get input file path.
    pub fn get_input_filepath(&self) -> PathBuf;

    /// Save an image to a file.
    /// If path is None, the original file will be overwritten.
    pub fn save_image(&mut self, path: Option<&str>) -> Result<SaveStatus, RusimgError>;
}
```

#### Rect

Struct ``Rect`` is used to specify the crop area.  
``rusimg::RusImg.trim_rect()`` needs a ``Rect`` object to specify the crop area.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
```

#### ImgSize

Struct ``ImgSize`` is used to get the image size.  
``rusimg::RusImg.get_image_size()``, ``rusimg::RusImg.resize()``, ``rusimg::RusImg.trim()``, and ``rusimg::RusImg.trim_rect()`` return this struct.

```rust
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub struct ImgSize {
    pub width: usize,
    pub height: usize,
}
```

#### SaveStatus

Struct ``SaveStatus`` is used for tracking the status of saving an image.  
It contains the output file path, the file size before saving, and the file size after saving.  
If the image has compression, the file size after saving will be different from the file size before saving.  
``rusimg::RusImg.save_image()`` returns this enum.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct SaveStatus {
    pub output_path: Option<PathBuf>,
    pub before_filesize: u64,
    pub after_filesize: Option<u64>,
}
```

### Enum

#### Extension

Enum ``Extension`` indicates the file extension.  
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
