use std::path::{Path, PathBuf};
use std::fs;
use std::fmt;
use glob::glob;
use parse::ArgStruct;
use rusimg::{RusimgError, RusimgStatus};
use colored::*;

mod parse;
mod rusimg;

pub enum ProcessingError {
    RusimgError(RusimgError),
    IOError(String),
    ArgError(String),
}
impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessingError::RusimgError(e) => write!(f, "{}", e.to_string()),
            ProcessingError::IOError(e) => write!(f, "{}", e),
            ProcessingError::ArgError(e) => write!(f, "{}", e),
        }
    }
}

fn get_files_in_dir(dir_path: &PathBuf, recursive: bool) -> Result<Vec<PathBuf>, String> {
    let mut files = fs::read_dir(&dir_path).expect("cannot read directory");
    let mut ret = Vec::new();

    while let Some(file) = files.next() {
        let dir_entry = file;
        match dir_entry {
            Ok(dir_entry) => {
                let path = dir_entry.path();
                // recursive に探索
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

fn get_files_by_wildcard(source_path: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut ret = Vec::new();
    for entry in glob(source_path.to_str().unwrap()).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                // 画像形式であればファイルリストに追加
                if get_extension(&path).is_ok() {
                    ret.push(path);
                }
            },
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(ret)
}

// 拡張子に.を含まない
fn convert_str_to_extension(extension_str: &str) -> Result<rusimg::Extension, RusimgError> {
    match extension_str {
        "bmp" => Ok(rusimg::Extension::Bmp),
        "jpg" | "jpeg" | "jfif" => Ok(rusimg::Extension::Jpeg),
        "png" => Ok(rusimg::Extension::Png),
        "webp" => Ok(rusimg::Extension::Webp),
        _ => Err(RusimgError::UnsupportedFileExtension),
    }
}

// 拡張子に.を含む
fn get_extension(path: &Path) -> Result<rusimg::Extension, RusimgError> {
    let path = path.to_str().ok_or(RusimgError::FailedToConvertPathToString)?.to_ascii_lowercase();
    match Path::new(&path).extension().and_then(|s| s.to_str()) {
        Some("bmp") => Ok(rusimg::Extension::Bmp),
        Some("jpg") | Some("jpeg") | Some("jfif") => Ok(rusimg::Extension::Jpeg),
        Some("png") => Ok(rusimg::Extension::Png),
        Some("webp") => Ok(rusimg::Extension::Webp),
        _ => {
            Err(RusimgError::UnsupportedFileExtension)
        },
    }
}

// 保存先などの表示
fn save_print(before_path: PathBuf, after_path: Option<PathBuf>, before_size: u64, after_size: Option<u64>) {
    match (after_path, after_size) {
        (Some(after_path), Some(after_size)) => {
            if before_path == after_path {
                println!("{}: {}", "Overwrite".bold(), before_path.display());
                println!("File Size: {} -> {} ({:.1}%)", before_size, after_size, (after_size as f64 / before_size as f64) * 100.0);
            }
            else if get_extension(before_path.as_path()) != get_extension(after_path.as_path()) {
                println!("{}: {} -> {}", "Convert".bold(), before_path.display(), after_path.display());
                println!("File Size: {} -> {} ({:.1}%)", before_size, after_size, (after_size as f64 / before_size as f64) * 100.0);
            }
            else {
                println!("{}: {} -> {}", "Move".bold(), before_path.display(), after_path.display());
                println!("File Size: {} -> {} ({:.1}%)", before_size, after_size, (after_size as f64 / before_size as f64) * 100.0);
            }
        },
        (_, _) => {
            return;
        },
    }
}

fn process(args: &ArgStruct, image_file_path: &PathBuf) -> Result<rusimg::RusimgStatus, ProcessingError> {
    let rierr = |e: RusimgError| ProcessingError::RusimgError(e);
    let ioerr = |e: std::io::Error| ProcessingError::IOError(e.to_string());
    let argerr = |e: String| ProcessingError::ArgError(e);

    // ファイルの上書き確認オプション
    let file_overwrite_ask = match (args.yes, args.no) {
        (true, false) => Some(rusimg::FileOverwriteAsk::YesToAll),
        (false, true) => Some(rusimg::FileOverwriteAsk::NoToAll),
        (false, false) => Some(rusimg::FileOverwriteAsk::AskEverytime),
        (true, true) => None,
    };
    let file_overwrite_ask = if let Some(ref _c) = file_overwrite_ask {
        file_overwrite_ask.unwrap()
    }
    else {
        return Err(argerr("Cannot specify both --yes and --no.".to_string()))?;
    };

    // ファイルを開く
    let mut image = rusimg::imgprocessor::do_open_image(&image_file_path).map_err(rierr)?;

    // --convert -> 画像形式変換
    if let Some(ref c) = args.destination_extension {
        let extension = convert_str_to_extension(&c).map_err(rierr)?;

        // 変換
        image = rusimg::imgprocessor::do_convert(&mut image, &extension).map_err(rierr)?;
    }

    // --trim -> トリミング
    if let Some(trim) = args.trim {
        // トリミング
        let before_size = rusimg::imgprocessor::do_get_image_size(&image).map_err(rierr)?;
        let trimmed_size = rusimg::imgprocessor::do_trim(&mut image, (trim.0.0, trim.0.1), (trim.1.0, trim.1.1)).map_err(rierr)?;
        if before_size != trimmed_size {
            println!("Trim: {}x{} -> {}x{}", before_size.width, before_size.height, trimmed_size.width, trimmed_size.height);
        }
    }

    // --resize -> リサイズ
    if let Some(resize) = args.resize {
        // リサイズ
        let before_size = rusimg::imgprocessor::do_get_image_size(&image).map_err(rierr)?;
        let after_size = rusimg::imgprocessor::do_resize(&mut image, resize).map_err(rierr)?;
        println!("Resize: {}x{} -> {}x{}", before_size.width, before_size.height, after_size.width, after_size.height);
    }

    // --grayscale -> グレースケール
    if args.grayscale {
        // グレースケール
        rusimg::imgprocessor::do_grayscale(&mut image).map_err(rierr)?;
        println!("Grayscale: Done.");
    }

    // --quality -> 圧縮
    if let Some(q) = args.quality {
        // 圧縮
        rusimg::imgprocessor::do_compress(&mut image.data, &image.extension, Some(q)).map_err(rierr)?;
        println!("Compress: Done.");
    }

    // 出力
    let output_path = match &args.destination_path {
        Some(path) => Some(path.clone()),
        None => None,
    };
    let (save_status, saved_filepath, opened_filepath, before_size, after_size)
         = rusimg::imgprocessor::do_save_image(output_path, &mut image.data, &image.extension, file_overwrite_ask).map_err(rierr)?;
    save_print(opened_filepath, saved_filepath.clone(), before_size, after_size);

    // --delete -> 元ファイルの削除 (optinal)
    if let Some(ref saved_filepath) = saved_filepath {
        if args.delete && image_file_path != saved_filepath {
            fs::remove_file(&image_file_path).map_err(ioerr)?;
        }
    }

    // 表示
    if args.view {
        rusimg::imgprocessor::do_view(&mut image).map_err(rierr)?;
    }

    Ok(save_status)
}

fn main() -> Result<(), String> {
    // 引数のパース
    let args = parse::parser();

    // 作業ディレクトリの指定（default: current dir）
    let source_paths = args.souce_path.clone().or(Some(vec![PathBuf::from(".")])).unwrap();
    let mut image_files = Vec::new();
    for source_path in source_paths {
        let image_files_temp = if source_path.is_dir() {
            get_files_in_dir(&source_path, args.recursive)?
        }
        else {
            get_files_by_wildcard(&source_path)?
        };
        for image_file in image_files_temp {
            image_files.push(image_file);
        }
    }

    // 検出した画像ファイルパスの表示
    let total_image_count = image_files.len();
    println!("{}", format!("🔎 {} images are detected.", total_image_count).bold());
    /*
    let mut str_width_count = 0;
    for image_file_path in &image_files {
        if str_width_count == 0 {
            print!("  ");
            print!("{}\t", image_file_path.to_str().unwrap());
        }
        else {
            println!("{}", image_file_path.to_str().unwrap());
            str_width_count = 0;
        }
    }
    println!();
    */

    // 各画像に対する処理
    let mut error_count = 0;
    let mut count = 0;
    for image_file_path in image_files {
        count = count + 1;
        let processing_str = format!("[{}/{}] Processing: {}", count, total_image_count, &Path::new(&image_file_path).file_name().unwrap().to_str().unwrap());
        println!("{}", processing_str.yellow().bold());

        match process(&args, &image_file_path) {
            Ok(status) => {
                match status {
                    rusimg::RusimgStatus::Success => println!("{}", "Success.".green().bold()),
                    rusimg::RusimgStatus::Cancel => println!("{}", "Canceled.".yellow().bold()),
                    _ => {},
                }
            },
            Err(e) => {
                println!("{}: {}", "Error".red(), e.to_string());
                error_count = error_count + 1;
            },
        }
    }

    if error_count > 0 {
        println!("\n✅ {} images are processed.", total_image_count - error_count);
        println!("❌ {} images are failed to process.", error_count);
    }
    else {
        println!("\n✅ All images are processed.");
    }

    Ok(())
}
