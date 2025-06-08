use std::path::{Path, PathBuf};
use std::fs;
use std::fmt;
use glob::glob;
use image::DynamicImage;
use colored::*;

use librusimg::{RusImg, RusimgError};
pub mod parse;
use parse::ArgStruct;

// Error types
type ErrorOccuredFilePath = String;
type ErrorMessage = std::io::Error;
/// Error structure containing the error and the file path where the error occurred.
pub struct ErrorStruct<T> {
    pub error: T,
    pub filepath: ErrorOccuredFilePath,
}
/// ProcessingError is an error type that occurs during image processing.
pub enum ProcessingError {
    RusimgError(ErrorStruct<RusimgError>),
    IOError(ErrorStruct<ErrorMessage>),
    FailedToViewImage(String),
    FailedToConvertExtension(ErrorStruct<ErrorMessage>),
}
impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessingError::RusimgError(e) => write!(f, "{}", e.error),
            ProcessingError::IOError(e) => write!(f, "{}", e.error),
            ProcessingError::FailedToViewImage(s) => write!(f, "Failed to view image: {}", s),
            ProcessingError::FailedToConvertExtension(e) => write!(f, "Failed to convert extension: {}", e.error),
        }
    }
}

// result status
/// FileOverwriteAsk is an enum that represents the status of whether to overwrite a file.
/// This is used to determine whether to overwrite a file when it already exists.
/// - YesToAll: Overwrite all files without asking. This is used when the --yes option is specified.
/// - NoToAll: Skip all files without asking. This is used when the --no option is specified.
/// - AskEverytime: Ask every time.
#[derive(Debug, Clone, PartialEq)]
pub enum FileOverwriteAsk {
    YesToAll,
    NoToAll,
    AskEverytime,
}
/// ExistsCheckResult is an enum that represents the result of checking whether a file exists.
/// - AllOverwrite: Overwrite all files without asking. This is used when the --yes option is specified.
/// - AllSkip: Skip all files without asking. This is used when the --no option is specified.
/// - NeedToAsk: Ask every time.
/// - NoProblem: No problem. This means that the file does not exist.
#[derive(Debug, Clone, PartialEq)]
pub enum ExistsCheckResult {
    AllOverwrite,
    AllSkip,
    NeedToAsk,
    NoProblem,
}

/// ConvertResult is a structure that represents the result of converting an image.
/// This structure will be used to display the result of the conversion.
/// - before_extension: The extension of the image before conversion.
/// - after_extension: The extension of the image after conversion.
pub struct ConvertResult {
    pub before_extension: librusimg::Extension,
    pub after_extension: librusimg::Extension,
}
/// TrimResult is a structure that represents the result of trimming an image.
/// This structure will be used to display the result of the trimming.
/// - before_size: The size of the image before trimming.
/// - after_size: The size of the image after trimming.
pub struct TrimResult {
    pub before_size: librusimg::ImgSize,
    pub after_size: librusimg::ImgSize,
}
/// ResizeResult is a structure that represents the result of resizing an image.
/// This structure will be used to display the result of the resizing.
/// - before_size: The size of the image before resizing.
/// - after_size: The size of the image after resizing.
pub struct ResizeResult {
    pub before_size: librusimg::ImgSize,
    pub after_size: librusimg::ImgSize,
}
/// GrayscaleResult is a structure that represents the result of converting an image to grayscale.
/// This structure will be used to display the result of the grayscale conversion.
/// - status: The status of the grayscale conversion.
pub struct GrayscaleResult {
    pub status: bool,
}
/// CompressResult is a structure that represents the result of compressing an image.
/// This structure will be used to display the result of the compression.
/// - status: The status of the compression.
pub struct CompressResult {
    pub status: bool,
}

/// Get the list of files in the directory.
/// This function used to get the list of image files in the directory when the --source option is specified with a directory path.
/// - dir_path: The path to the directory.
/// - recursive: Whether to search recursively.
pub fn get_files_in_dir(dir_path: &PathBuf, recursive: bool) -> Result<Vec<PathBuf>, String> {
    let mut files = fs::read_dir(&dir_path).expect("cannot read directory");
    let mut ret = Vec::new();

    while let Some(file) = files.next() {
        let dir_entry = file;
        match dir_entry {
            Ok(dir_entry) => {
                let path = dir_entry.path();
                // recursively search the directory
                if path.is_dir() && recursive {
                    let mut files = get_files_in_dir(&path, recursive)?;
                    ret.append(&mut files);
                }
                else {
                    let file_name = dir_entry.file_name().into_string().expect("cannot convert file name");
                    if get_extension(&Path::new(&file_name)).is_ok() {
                        ret.push(Path::new(&dir_path).join(&file_name));
                    }
                }
            },
            Err(e) => {
                println!("cannot read a directory entry: {}", e.to_string());
                continue;
            },
        }
    }
    Ok(ret)
}

/// Get the list of files by wildcard.
/// This function used to get the list of image files by wildcard when the --source option is specified with a wildcard pattern.
pub fn get_files_by_wildcard(source_path: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut ret = Vec::new();
    for entry in glob(source_path.to_str().unwrap()).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                // If the file is an image format, add it to the file list.
                if get_extension(&path).is_ok() {
                    ret.push(path);
                }
            },
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(ret)
}

/// Is saving the image required?
pub fn is_save_required(args: &ArgStruct) -> bool {
    if args.destination_extension.is_some() || args.trim.is_some() || args.resize.is_some() || args.grayscale || args.quality.is_some() {
        return true;
    }
    if args.destination_path.is_some() {
        return true;
    }
    false
}

/// Get destination's extension.
pub fn get_destination_extension(source_filepath: &PathBuf, dest_extension: &Option<librusimg::Extension>) -> librusimg::Extension {
    if let Some(extension) = dest_extension {
        extension.clone()
    }
    else {
        // If the destination extension is not specified, use the same extension as the source file.
        get_extension(source_filepath.as_path()).unwrap_or(librusimg::Extension::Png)
    }
}

/// Convert a string to an image extension.
pub fn convert_str_to_extension(extension_str: &str) -> Result<librusimg::Extension, RusimgError> {
    match extension_str {
        "bmp" => Ok(librusimg::Extension::Bmp),
        "jpg" => Ok(librusimg::Extension::Jpg),
        "jpeg" | "jfif" => Ok(librusimg::Extension::Jpeg),
        "png" => Ok(librusimg::Extension::Png),
        "webp" => Ok(librusimg::Extension::Webp),
        _ => Err(RusimgError::UnsupportedFileExtension),
    }
}

/// Get the extension of the file.
pub fn get_extension(path: &Path) -> Result<librusimg::Extension, RusimgError> {
    let path = path.to_str().ok_or(RusimgError::FailedToConvertPathToString)?.to_ascii_lowercase();
    match Path::new(&path).extension().and_then(|s| s.to_str()) {
        Some("bmp") => Ok(librusimg::Extension::Bmp),
        Some("jpg") => Ok(librusimg::Extension::Jpg),
        Some("jpeg") | Some("jfif") => Ok(librusimg::Extension::Jpeg),
        Some("png") => Ok(librusimg::Extension::Png),
        Some("webp") => Ok(librusimg::Extension::Webp),
        _ => {
            Err(RusimgError::UnsupportedFileExtension)
        },
    }
}

/// Determine the output path.
pub fn get_output_path(input_path: &PathBuf, output_path: &Option<PathBuf>, double_extension: bool, destination_append_name: &Option<String>, extension: &librusimg::Extension) -> PathBuf {
    let extension = if double_extension {
        format!("{}.{}", input_path.extension().unwrap().to_str().unwrap(), extension.to_string())
    }
    else {
        extension.to_string()
    };
    let mut output_path = match output_path {
        //Some(path) => path.clone(),                                                             // If --output is specified, use it
        Some(path) => {
            // Is the path a file or a directory?
            if path.is_dir() {
                // If the path is a directory, use the input file name as the output file name.
                let mut output_path_tmp = path.clone();
                output_path_tmp.push(input_path.file_name().unwrap());
                output_path_tmp.set_extension(&extension.to_string());
                output_path_tmp
            }
            else {
                // Otherwise, if an extension is specified, use it as the output file name.
                if path.extension().is_some() {
                    path.clone()
                }
                else {
                    // If the extension is not specified, use the input file name as the output file name.
                    let mut output_path_tmp = path.clone();
                    output_path_tmp.push(input_path.file_name().unwrap());
                    output_path_tmp.set_extension(&extension.to_string());
                    // Make the directory if it does not exist.
                    if !output_path_tmp.parent().unwrap().exists() {
                        fs::create_dir_all(output_path_tmp.parent().unwrap()).unwrap();
                    }
                    output_path_tmp
                }
            }
        }
        None => Path::new(input_path).with_extension(&extension.to_string()),       // If not, use the input filepath as the input file
    };
    // If append_name is specified, add it to the file name.
    if let Some(append_name) = &destination_append_name {
        let mut output_path_tmp = output_path.file_stem().unwrap().to_str().unwrap().to_string();
        output_path_tmp.push_str(append_name);
        output_path_tmp.push_str(".");
        output_path_tmp.push_str(&extension.to_string());
        output_path = PathBuf::from(output_path_tmp);
    }
    output_path
}

/// Check if the file exists.
/// If the file exists, check if it should be overwritten.
pub fn check_file_exists(path: &PathBuf, file_overwrite_ask: &FileOverwriteAsk) -> ExistsCheckResult {
    if Path::new(path).exists() {
        println!("The image file \"{}\" already exists.", path.display().to_string().yellow().bold());
        match file_overwrite_ask {
            FileOverwriteAsk::YesToAll => {
                return ExistsCheckResult::AllOverwrite;
            },
            FileOverwriteAsk::NoToAll => {
                return ExistsCheckResult::AllSkip;
            },
            FileOverwriteAsk::AskEverytime => {
                return ExistsCheckResult::NeedToAsk;
            },
        }
    }
    return ExistsCheckResult::NoProblem;
}

/// Show the image in the terminal using viuer.
/// Read the image data from memory and display it.
pub fn view(image: &DynamicImage) -> Result<(), ProcessingError> {
    let width = image.width();
    let height = image.height();
    let conf_width = width as f64 / std::cmp::max(width, height) as f64 * 100 as f64;
    let conf_height = height as f64 / std::cmp::max(width, height) as f64 as f64 * 50 as f64;
    let conf = viuer::Config {
        absolute_offset: false,
        width: Some(conf_width as u32),
        height: Some(conf_height as u32),    
        ..Default::default()
    };
    
    let result = viuer::print(&image, &conf);
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(ProcessingError::FailedToViewImage(e.to_string())),
    }
}

/// Convert an image.
pub fn process_convert<C: Fn(RusimgError) -> ProcessingError>(extension: &Option<librusimg::Extension>, image: &mut RusImg, rierr: C) -> Result<Option<ConvertResult>, ProcessingError> {
    if let Some(extension) = extension {
        let before_extension = image.get_extension();

        // 変換
        image.convert(&extension).map_err(rierr)?;

        Ok(Some(ConvertResult {
            before_extension: before_extension,
            after_extension: extension.clone(),
        }))
    }
    else {
        Err(ProcessingError::FailedToConvertExtension(ErrorStruct {
            error: std::io::Error::new(std::io::ErrorKind::Other, "Failed to convert extension."),
            filepath: image.get_input_filepath().map_err(rierr)?.to_str().unwrap().to_string(),
        }))
    }
}

/// Trim an image.
pub fn process_trim<C: Fn(RusimgError) -> ProcessingError>(image: &mut RusImg, trim: librusimg::Rect, rierr: C) -> Result<Option<TrimResult>, ProcessingError> {
    // トリミング
    let before_size = image.get_image_size().map_err(&rierr)?;
    let after_size = image.trim_rect(trim).map_err(&rierr)?;

    Ok(Some(TrimResult {
        before_size: before_size,
        after_size: after_size,
    }))
}

/// Resize an image.
pub fn process_resize<C: Fn(RusimgError) -> ProcessingError>(image: &mut RusImg, resize: f32, rierr: C) -> Result<Option<ResizeResult>, ProcessingError> {
    let before_size = image.get_image_size().map_err(&rierr)?;
    let after_size = image.resize(resize).map_err(&rierr)?;
    
    Ok(Some(ResizeResult {
        before_size: before_size,
        after_size: after_size,
    }))
}

/// Convert an image to grayscale.
pub fn process_grayscale<C: Fn(RusimgError) -> ProcessingError>(image: &mut RusImg, rierr: C) -> Result<Option<GrayscaleResult>, ProcessingError> {
    image.grayscale().map_err(rierr)?;
    
    Ok(Some(GrayscaleResult {
        status: true,
    }))
}

/// Compress an image.
pub fn process_compress<C: Fn(RusimgError) -> ProcessingError>(image: &mut RusImg, quality: Option<f32>, rierr: C) -> Result<Option<CompressResult>, ProcessingError> {
    if let Some(q) = quality {
        image.compress(Some(q)).map_err(rierr)?;
    }
    
    Ok(Some(CompressResult {
        status: true,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;
    use image::{ImageBuffer, Rgb, DynamicImage};
    use librusimg::RusImg;
    use librusimg::Extension;

    fn generate_test_image(filename: &str, width: u32, height: u32) {
        let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
        for x in 0..width {
            for y in 0..height {
                let r = (x * 3) as u8;
                let g = (y * 5) as u8;
                let b = (x * y) as u8;
                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        let mut test_image = RusImg::new(&Extension::Png, DynamicImage::ImageRgb8(img.clone())).unwrap();
        test_image.save_image(Some(filename)).unwrap();
    }

    #[test]
    fn test_get_files_in_dir() {
        let dir_path = PathBuf::from("test_dir1");
        fs::create_dir_all(&dir_path).unwrap();
        generate_test_image("test_dir1/test_image1.png", 100, 100);
        generate_test_image("test_dir1/test_image2.jpg", 200, 200);
        generate_test_image("test_dir1/test_image3.bmp", 300, 300);

        let files = get_files_in_dir(&dir_path, false).unwrap();
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.ends_with("test_image1.png")));
        assert!(files.iter().any(|f| f.ends_with("test_image2.jpg")));
        assert!(files.iter().any(|f| f.ends_with("test_image3.bmp")));

        fs::remove_dir_all(&dir_path).unwrap();
    }

    #[test]
    fn test_get_files_by_wildcard() {
        let wildcard_path = PathBuf::from("test_dir2/*.png");
        fs::create_dir_all("test_dir2").unwrap();
        generate_test_image("test_dir2/test_image1.png", 100, 100);
        generate_test_image("test_dir2/test_image2.jpg", 200, 200);
        generate_test_image("test_dir2/test_image3.bmp", 300, 300);

        let files = get_files_by_wildcard(&wildcard_path).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.iter().any(|f| f.ends_with("test_image1.png")));

        fs::remove_dir_all("test_dir2").unwrap();
    }

    #[test]
    fn test_get_extension() {
        let path = PathBuf::from("test_image.png");
        let ext = get_extension(&path).unwrap();
        assert_eq!(ext, librusimg::Extension::Png);

        let path = PathBuf::from("test_image.jpg");
        let ext = get_extension(&path).unwrap();
        assert_eq!(ext, librusimg::Extension::Jpg);

        let path = PathBuf::from("test_image.bmp");
        let ext = get_extension(&path).unwrap();
        assert_eq!(ext, librusimg::Extension::Bmp);

        let path = PathBuf::from("test_image.webp");
        let ext = get_extension(&path).unwrap();
        assert_eq!(ext, librusimg::Extension::Webp);
    }

    #[test]
    fn test_convert_str_to_extension() {
        let ext = convert_str_to_extension("jpg").unwrap();
        assert_eq!(ext, librusimg::Extension::Jpg);

        let ext = convert_str_to_extension("jpeg").unwrap();
        assert_eq!(ext, librusimg::Extension::Jpeg);

        let ext = convert_str_to_extension("png").unwrap();
        assert_eq!(ext, librusimg::Extension::Png);

        let ext = convert_str_to_extension("bmp").unwrap();
        assert_eq!(ext, librusimg::Extension::Bmp);

        let ext = convert_str_to_extension("webp").unwrap();
        assert_eq!(ext, librusimg::Extension::Webp);
    }

    #[test]
    fn test_get_destination_extension() {
        let source_path = PathBuf::from("test_image.png");
        let dest_extension = get_destination_extension(&source_path, &Some(librusimg::Extension::Jpg));
        assert_eq!(dest_extension, librusimg::Extension::Jpg);

        let dest_extension = get_destination_extension(&source_path, &None);
        assert_eq!(dest_extension, librusimg::Extension::Png);
    }

    #[test]
    fn test_get_output_path() {
        let input_path = PathBuf::from("test_image.png");
        let output_path = get_output_path(&input_path, &None, false, &None, &librusimg::Extension::Jpg);
        assert_eq!(output_path.to_str().unwrap(), "test_image.jpg");

        let output_path = get_output_path(&input_path, &Some(PathBuf::from("output_dir3")), false, &None, &librusimg::Extension::Jpg);
        assert_eq!(output_path, PathBuf::from("output_dir3").join("test_image.jpg"));

        let output_path = get_output_path(&input_path, &Some(PathBuf::from("output_dir3/test_image2.jpg")), false, &None, &librusimg::Extension::Jpg);
        assert_eq!(output_path, PathBuf::from("output_dir3").join("test_image2.jpg"));

        fs::remove_dir_all("output_dir3").unwrap_or(());
    }

    #[test]
    fn test_check_file_exists() {
        let path = PathBuf::from("test_image.png");
        fs::write(&path, b"test").unwrap();
        let result = check_file_exists(&path, &FileOverwriteAsk::NoToAll);
        assert_eq!(result, ExistsCheckResult::AllSkip);
        let result = check_file_exists(&path, &FileOverwriteAsk::YesToAll);
        assert_eq!(result, ExistsCheckResult::AllOverwrite);
        let result = check_file_exists(&path, &FileOverwriteAsk::AskEverytime);
        assert_eq!(result, ExistsCheckResult::NeedToAsk);
        let not_exists_result = check_file_exists(&PathBuf::from("not_exists.png"), &FileOverwriteAsk::NoToAll);
        assert_eq!(not_exists_result, ExistsCheckResult::NoProblem);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_parser_default() {
        let args = parse::parser().unwrap();
        assert_eq!(args.souce_path, None);
        assert_eq!(args.destination_path, None);
        assert_eq!(args.destination_append_name, None);
        assert_eq!(args.destination_extension, None);
        assert_eq!(args.resize, None);
        assert_eq!(args.trim, None);
        assert_eq!(args.grayscale, false);
        assert_eq!(args.quality, None);
        assert_eq!(args.double_extension, false);
        assert_eq!(args.view, false);
        assert_eq!(args.yes, false);
        assert_eq!(args.no, false);
        assert_eq!(args.delete, false);
    }

    #[test]
    fn test_parser_error_cases() {
        // trim area is invalid
        match parse::check_trim_format("10x10+20x20") {
            Ok(trim) => assert_eq!(trim, librusimg::Rect { x: 10, y: 10, w: 20, h: 20 }),
            Err(_) => panic!("Trim area is invalid."),
        }
        match parse::check_trim_format("10") {
            Ok(_) => panic!("Trim area is valid."),
            Err(_) => {},
        }
        match parse::check_trim_format("10x10") {
            Ok(_) => panic!("Trim area is valid."),
            Err(_) => {},
        }
        match parse::check_trim_format("10+10+20+20") {
            Ok(_) => panic!("Trim area is valid."),
            Err(_) => {},
        }
        // resize range is invalid
        match parse::check_resize_range(Some(-1.0)) {
            true => panic!("Resize range is valid."),
            false => {},
        }
        match parse::check_resize_range(Some(0.0)) {
            true => panic!("Resize range is valid."),
            false => {},
        }
        // quality range is invalid
        match parse::check_quality_range(Some(110.0)) {
            true => panic!("Quality range is valid."),
            false => {},
        }
        match parse::check_quality_range(Some(-1.0)) {
            true => panic!("Quality range is valid."),
            false => {},
        }
        match parse::check_quality_range(Some(50.0)) {
            true => {},
            false => panic!("Quality range is invalid."),
        }
        // threads is invalid
        match parse::check_threads_range(0) {
            true => panic!("Threads range is valid."),
            false => {},
        }
    }

    #[test]
    fn test_convert_and_save() {
        let input_path = PathBuf::from("test_image.png");
        generate_test_image(input_path.to_str().unwrap(), 100, 100);
        let output_path = PathBuf::from("test_image_converted.jpg");
        let mut image = librusimg::RusImg::open(&input_path).unwrap();
        image.convert(&librusimg::Extension::Jpg).unwrap();
        image.save_image(Some(output_path.to_str().unwrap())).unwrap();
        assert!(output_path.exists(), "Output image does not exist: {}", output_path.display());
        fs::remove_file(&input_path).unwrap_or(());
        fs::remove_file(&output_path).unwrap_or(());
    }

    #[test]
    fn test_resize_and_save() {
        let input_path = PathBuf::from("test_image.png");
        generate_test_image(input_path.to_str().unwrap(), 100, 100);
        let output_path = PathBuf::from("test_image_resized.jpg");
        let mut image = librusimg::RusImg::open(&input_path).unwrap();
        image.resize(50.0).unwrap();
        image.save_image(Some(output_path.to_str().unwrap())).unwrap();
        assert!(output_path.exists(), "Output image does not exist: {}", output_path.display());
        assert!(image.get_image_size().unwrap().width == 50, "Image size is not resized: {}", image.get_image_size().unwrap().width);
        assert!(image.get_image_size().unwrap().height == 50, "Image size is not resized: {}", image.get_image_size().unwrap().height);
        fs::remove_file(&input_path).unwrap_or(());
        fs::remove_file(&output_path).unwrap_or(());
    }

    #[test]
    fn test_trim_and_save() {
        let input_path = PathBuf::from("test_image.png");
        generate_test_image(input_path.to_str().unwrap(), 100, 100);
        let output_path = PathBuf::from("test_image_trimmed.jpg");
        let mut image = librusimg::RusImg::open(&input_path).unwrap();
        image.trim_rect(librusimg::Rect { x: 10, y: 10, w: 50, h: 50 }).unwrap();
        image.save_image(Some(output_path.to_str().unwrap())).unwrap();
        assert!(output_path.exists(), "Output image does not exist: {}", output_path.display());
        assert!(image.get_image_size().unwrap().width == 50, "Image size is not trimmed: {}", image.get_image_size().unwrap().width);
        assert!(image.get_image_size().unwrap().height == 50, "Image size is not trimmed: {}", image.get_image_size().unwrap().height);
        fs::remove_file(&input_path).unwrap_or(());
        fs::remove_file(&output_path).unwrap_or(());
    }

    #[test]
    fn test_grayscale_and_save() {
        let input_path = PathBuf::from("test_image.png");
        generate_test_image(input_path.to_str().unwrap(), 100, 100);
        let output_path = PathBuf::from("test_image_grayscale.jpg");
        let mut image = librusimg::RusImg::open(&input_path).unwrap();
        image.grayscale().unwrap();
        image.save_image(Some(output_path.to_str().unwrap())).unwrap();
        assert!(output_path.exists(), "Output image does not exist: {}", output_path.display());
        fs::remove_file(&input_path).unwrap_or(());
        fs::remove_file(&output_path).unwrap_or(());
    }

    #[test]
    #[ignore] // This test requires the machine to have the rusimg binary installed. Run with `cargo test -- --ignored`.
    fn run_test() {
        use std::process::Command;

        // Create a test directory and test image.
        let test_dir = PathBuf::from("test_dir3");
        fs::create_dir_all(&test_dir).unwrap();

        let image_files = vec![
            "test_dir3/test_image1.png",
            "test_dir3/test_image2.jpg",
            "test_dir3/test_image3.bmp",
        ];
        let original_size = librusimg::ImgSize { width: 100, height: 100 };
        for image_file in &image_files {
            generate_test_image(image_file, original_size.width as u32, original_size.height as u32);
        }

        let mut cmd = Command::new("rusimg");
        cmd.arg("-i")
            .arg(test_dir.clone())
            .arg("-o")
            .arg("test_dir3/output_dir")
            .arg("-c")
            .arg("webp")
            .arg("-r")
            .arg("80")
            .arg("-t")
            .arg("10x10+20x20")
            .arg("-g")
            .arg("-q")
            .arg("80.0")
            .arg("-d")
            .arg("-v")
            .arg("-y")
            .arg("-D");
        let assert = cmd.output().unwrap();
        assert!(assert.status.success(), "Command failed: {}", String::from_utf8_lossy(&assert.stderr));

        // Check output images
        let image_files_output = vec![
            "test_dir3/output_dir/test_image1.png.webp",
            "test_dir3/output_dir/test_image2.jpg.webp",
            "test_dir3/output_dir/test_image3.bmp.webp",
        ];
        for (image_file_output, image_file_input) in image_files_output.iter().zip(image_files.iter()) {
            let image_file_output = PathBuf::from(image_file_output);
            let image_file_input = PathBuf::from(image_file_input);
            // Is the output image created?
            let output_image = PathBuf::from("test_dir3/output_dir").join(image_file_output.file_name().unwrap());
            assert!(output_image.exists(), "Output image does not exist: {}", output_image.display());
            // Is the output image extension webp?
            let output_extension = get_extension(&output_image).unwrap();
            assert_eq!(output_extension, librusimg::Extension::Webp, "Output image extension is not webp: {}", output_image.display());
            // Is the original image deleted?
            assert!(!image_file_input.exists(), "Original image is not deleted: {}", image_file_input.display());
            // Is the output image size smaller than the original image size?
            let mut output_image = RusImg::open(&output_image).unwrap();
            let output_size = output_image.get_image_size().unwrap();
            assert!(output_size.width < original_size.width, "Output image size is not smaller than original image size: {} -> {}", original_size.width, output_size.width);
            assert!(output_size.height < original_size.height, "Output image size is not smaller than original image size: {} -> {}", original_size.height, output_size.height);
            // Is the output image grayscale?
            let output_image = output_image.get_dynamic_image().unwrap();
            let output_image = output_image.grayscale().to_rgb8();
            let mut is_grayscale = true;
            for pixel in output_image.pixels() {
                if pixel[0] != pixel[1] || pixel[1] != pixel[2] {
                    is_grayscale = false;
                    break;
                }
            }
            assert!(is_grayscale, "Output image is not grayscale.");
        }

        // Clean up test directory and images
        fs::remove_dir_all(&test_dir).unwrap_or(());
    }
}
