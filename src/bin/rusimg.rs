use std::path::{Path, PathBuf};
use std::fs;
use std::fmt;
use std::io::{stdout, Write};
use glob::glob;
use image::DynamicImage;
use parse::ArgStruct;
use colored::*;

extern crate rusimg;
use rusimg::rusimg::RusimgError;
#[path = "./rusimg/parse.rs"]
mod parse;

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

#[derive(Debug, Clone, PartialEq)]
pub enum FileOverwriteAsk {
    YesToAll,
    NoToAll,
    AskEverytime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RusimgStatus {
    Success,
    Cancel,
    NotNeeded,
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
fn convert_str_to_extension(extension_str: &str) -> Result<rusimg::rusimg::Extension, RusimgError> {
    match extension_str {
        "bmp" => Ok(rusimg::rusimg::Extension::Bmp),
        "jpg" | "jpeg" | "jfif" => Ok(rusimg::rusimg::Extension::Jpeg),
        "png" => Ok(rusimg::rusimg::Extension::Png),
        "webp" => Ok(rusimg::rusimg::Extension::Webp),
        _ => Err(RusimgError::UnsupportedFileExtension),
    }
}

// 拡張子に.を含む
fn get_extension(path: &Path) -> Result<rusimg::rusimg::Extension, RusimgError> {
    let path = path.to_str().ok_or(RusimgError::FailedToConvertPathToString)?.to_ascii_lowercase();
    match Path::new(&path).extension().and_then(|s| s.to_str()) {
        Some("bmp") => Ok(rusimg::rusimg::Extension::Bmp),
        Some("jpg") | Some("jpeg") | Some("jfif") => Ok(rusimg::rusimg::Extension::Jpeg),
        Some("png") => Ok(rusimg::rusimg::Extension::Png),
        Some("webp") => Ok(rusimg::rusimg::Extension::Webp),
        _ => {
            Err(RusimgError::UnsupportedFileExtension)
        },
    }
}

// ファイルの存在チェック
fn check_file_exists(path: &PathBuf, file_overwrite_ask: &FileOverwriteAsk) -> bool {
    // ファイルの存在チェック
    // ファイルが存在する場合、上書きするかどうかを確認
    if Path::new(path).exists() {
        print!("The image file \"{}\" already exists.", path.display());
        match file_overwrite_ask {
            FileOverwriteAsk::YesToAll => {
                println!(" -> Overwrite it.");
                return true
            },
            FileOverwriteAsk::NoToAll => {
                println!(" -> Skip it.");
                return false
            },
            FileOverwriteAsk::AskEverytime => {},
        }

        print!(" Do you want to overwrite it? [y/N]: ");
        loop {
            stdout().flush().unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            if input.trim().to_ascii_lowercase() == "y" || input.trim().to_ascii_lowercase() == "yes" {
                return true;
            }
            else if input.trim().to_ascii_lowercase() == "n" || input.trim().to_ascii_lowercase() == "no" || input.trim() == "" {
                return false;
            }
            else {
                print!("Please enter y or n: ");
            }
        }
    }
    return true;
}

// 保存先などの表示
fn save_print(before_path: &PathBuf, after_path: &Option<PathBuf>, before_size: u64, after_size: Option<u64>) {
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

// viuer で表示
fn view(image: &DynamicImage) -> Result<(), RusimgError> {
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

    viuer::print(&image, &conf).map_err(|e| RusimgError::FailedToViewImage(e.to_string()))?;

    Ok(())
}

fn process(args: &ArgStruct, image_file_path: &PathBuf) -> Result<RusimgStatus, ProcessingError> {
    let rierr = |e: RusimgError| ProcessingError::RusimgError(e);
    let ioerr = |e: std::io::Error| ProcessingError::IOError(e.to_string());
    let argerr = |e: String| ProcessingError::ArgError(e);

    // ファイルの上書き確認オプション
    let file_overwrite_ask = match (args.yes, args.no) {
        (true, false) => Some(FileOverwriteAsk::YesToAll),
        (false, true) => Some(FileOverwriteAsk::NoToAll),
        (false, false) => Some(FileOverwriteAsk::AskEverytime),
        (true, true) => None,
    };
    let file_overwrite_ask = if let Some(ref _c) = file_overwrite_ask {
        file_overwrite_ask.unwrap()
    }
    else {
        return Err(argerr("Cannot specify both --yes and --no.".to_string()))?;
    };

    // ファイルを開く
    let mut image = rusimg::open_image(&image_file_path.to_str().unwrap()).map_err(rierr)?;

    // 保存が必要か？
    let mut save_required = false;

    // --convert -> 画像形式変換
    if let Some(ref c) = args.destination_extension {
        let extension = convert_str_to_extension(&c).map_err(rierr)?;
        println!("Convert: {} -> {}", image.extension.to_string(), extension.to_string());

        // 変換
        image.convert(extension).map_err(rierr)?;
        save_required = true;
    }

    // --trim -> トリミング
    if let Some(trim) = args.trim {
        // トリミング
        let before_size = image.get_image_size().map_err(rierr)?;
        let trimmed_size = image.trim(trim.0.0, trim.0.1, trim.1.0, trim.1.1).map_err(rierr)?;
        if before_size != trimmed_size {
            println!("Trim: {}x{} -> {}x{}", before_size.width, before_size.height, trimmed_size.width, trimmed_size.height);
            save_required = true;
        }
    }

    // --resize -> リサイズ
    if let Some(resize) = args.resize {
        // リサイズ
        let before_size = image.get_image_size().map_err(rierr)?;
        let after_size = image.resize(resize).map_err(rierr)?;
        println!("Resize: {}x{} -> {}x{}", before_size.width, before_size.height, after_size.width, after_size.height);
        save_required = true;
    }

    // --grayscale -> グレースケール
    if args.grayscale {
        // グレースケール
        image.grayscale().map_err(rierr)?;
        println!("Grayscale: Done.");
        save_required = true;
    }

    // --quality -> 圧縮
    if let Some(q) = args.quality {
        // 圧縮
        image.compress(Some(q)).map_err(rierr)?;
        println!("Compress: Done.");
        save_required = true;
    }

    // 出力
    let save_status = if save_required == true {
        println!("Save as {}...", image.extension.to_string());
        // 出力先パスを決定
        let mut output_path = match &args.destination_path {
            Some(path) => path.clone(),                                                             // If --output is specified, use it
            None => Path::new(&image.get_input_filepath()).with_extension(image.extension.to_string()),       // If not, use the input filepath as the input file
        };
        // append_name が指定されている場合、ファイル名に追加
        if let Some(append_name) = &args.destination_append_name {
            let mut output_path_tmp = output_path.file_stem().unwrap().to_str().unwrap().to_string();
            output_path_tmp.push_str(append_name);
            output_path_tmp.push_str(".");
            output_path_tmp.push_str(&image.extension.to_string());
            output_path = PathBuf::from(output_path_tmp);
        }

        // ファイルの存在チェック
        if !check_file_exists(&output_path, &file_overwrite_ask) {
            RusimgStatus::Cancel
        }
        else {
            // 保存
            let save_status = image.save_image(output_path.to_str()).map_err(rierr)?;
            // 保存先などの表示
            save_print(&image.get_input_filepath(), &save_status.output_path, 
                            save_status.before_filesize, save_status.after_filesize);

            // --delete -> 元ファイルの削除 (optinal)
            if let Some(ref saved_filepath) = save_status.output_path {
                if args.delete && image_file_path != saved_filepath {
                    fs::remove_file(&image_file_path).map_err(ioerr)?;
                }
            }
            RusimgStatus::Success
        }
    }
    else {
        RusimgStatus::NotNeeded
    };

    // 表示 (viuer)
    if args.view {
        view(&image.get_dynamic_image().map_err(rierr)?).map_err(rierr)?;
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
                    RusimgStatus::Success => println!("{}", "Success.".green().bold()),
                    RusimgStatus::Cancel => println!("{}", "Canceled.".yellow().bold()),
                    RusimgStatus::NotNeeded => {},
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
