use std::path::{Path, PathBuf};
use std::fs;
use std::fmt;
use std::io::{stdout, Write};
use glob::glob;
use image::DynamicImage;
use parse::ArgStruct;
use colored::*;
use std::thread;

use rusimg::RusimgError;
mod parse;

// error type
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

// result status
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

// process results
struct ConvertResult {
    before_extension: rusimg::Extension,
    after_extension: rusimg::Extension,
}
struct TrimResult {
    before_size: rusimg::ImgSize,
    after_size: rusimg::ImgSize,
}
struct ResizeResult {
    before_size: rusimg::ImgSize,
    after_size: rusimg::ImgSize,
}
struct GrayscaleResult {
    status: bool,
}
struct CompressResult {
    status: bool,
}
struct SaveResult {
    status: RusimgStatus,
    input_path: PathBuf,
    output_path: Option<PathBuf>,
    before_filesize: u64,
    after_filesize: Option<u64>,
    delete: bool,
}
struct ThreadResult {
    convert_result: Option<ConvertResult>,
    trim_result: Option<TrimResult>,
    resize_result: Option<ResizeResult>,
    grayscale_result: Option<GrayscaleResult>,
    compress_result: Option<CompressResult>,
    save_result: SaveResult,
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

fn process(args: &ArgStruct, image_file_path: &PathBuf) -> Result<ThreadResult, ProcessingError> {
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
    let mut image = rusimg::open_image(&image_file_path).map_err(rierr)?;

    // 保存が必要か？
    let mut save_required = false;

    // --convert -> 画像形式変換
    let convert_result = if let Some(ref c) = args.destination_extension {
        let extension = convert_str_to_extension(&c).map_err(rierr)?;
        println!("Convert: {} -> {}", image.extension.to_string(), extension.to_string());

        // 変換
        image.convert(&extension).map_err(rierr)?;
        save_required = true;

        Some(ConvertResult {
            before_extension: image.extension.clone(),
            after_extension: extension,
        })
    }
    else {
        None
    };

    // --trim -> トリミング
    let trim_result = if let Some(trim) = args.trim {
        // トリミング
        let before_size = image.get_image_size().map_err(rierr)?;
        let trimmed_size = image.trim(trim.0.0, trim.0.1, trim.1.0, trim.1.1).map_err(rierr)?;
        if before_size != trimmed_size {
            println!("Trim: {}x{} -> {}x{}", before_size.width, before_size.height, trimmed_size.width, trimmed_size.height);
            save_required = true;
        }

        Some(TrimResult {
            before_size: before_size,
            after_size: trimmed_size,
        })
    }
    else {
        None
    };

    // --resize -> リサイズ
    let resize_result = if let Some(resize) = args.resize {
        // リサイズ
        let before_size = image.get_image_size().map_err(rierr)?;
        let after_size = image.resize(resize).map_err(rierr)?;
        println!("Resize: {}x{} -> {}x{}", before_size.width, before_size.height, after_size.width, after_size.height);
        save_required = true;

        Some(ResizeResult {
            before_size: before_size,
            after_size: after_size,
        })
    }
    else {
        None
    };

    // --grayscale -> グレースケール
    let grayscale_result = if args.grayscale {
        // グレースケール
        image.grayscale().map_err(rierr)?;
        println!("Grayscale: Done.");
        save_required = true;

        Some(GrayscaleResult {
            status: true,
        })
    }
    else {
        None
    };

    // --quality -> 圧縮
    let compress_result = if let Some(q) = args.quality {
        // 圧縮
        image.compress(Some(q)).map_err(rierr)?;
        println!("Compress: Done.");
        save_required = true;

        Some(CompressResult {
            status: true,
        })
    }
    else {
        None
    };

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
            SaveResult {
                status: RusimgStatus::Cancel,
                input_path: image.get_input_filepath(),
                output_path: None,
                before_filesize: 0,
                after_filesize: None,
                delete: false,
            }
        }
        else {
            // 保存
            let save_status = image.save_image(output_path.to_str()).map_err(rierr)?;

            // --delete -> 元ファイルの削除 (optinal)
            let delete = if let Some(ref saved_filepath) = save_status.output_path {
                if args.delete && image_file_path != saved_filepath {
                    fs::remove_file(&image_file_path).map_err(ioerr)?;
                    true
                }
                else {
                    false
                }
            }
            else {
                false
            };

            SaveResult {
                status: RusimgStatus::Success,
                input_path: image.get_input_filepath(),
                output_path: save_status.output_path,
                before_filesize: save_status.before_filesize,
                after_filesize: save_status.after_filesize,
                delete: delete,
            }
        }
    }
    else {
        SaveResult {
            status: RusimgStatus::NotNeeded,
            input_path: image.get_input_filepath(),
            output_path: None,
            before_filesize: 0,
            after_filesize: None,
            delete: false,
        }
    };

    // 表示 (viuer)
    if args.view {
        view(&image.get_dynamic_image().map_err(rierr)?).map_err(rierr)?;
    }

    let thread_results = ThreadResult {
        convert_result: convert_result,
        trim_result: trim_result,
        resize_result: resize_result,
        grayscale_result: grayscale_result,
        compress_result: compress_result,
        save_result: save_status,
    };

    Ok(thread_results)
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
    let mut threads_vec = Vec::new();
    for image_file_path in image_files {
        count = count + 1;
        let processing_str = format!("[{}/{}] Processing: {}", count, total_image_count, &Path::new(&image_file_path).file_name().unwrap().to_str().unwrap());
        println!("{}", processing_str.yellow().bold());
        
        let args_clone = args.clone();
        let thread = std::thread::spawn(move || {
            process(&args_clone, &image_file_path)
        });
        threads_vec.push(thread);
    }
    for thread in threads_vec {
        match thread.join().unwrap() {
            Ok(thread_results) => {
                if let Some(convert_result) = thread_results.convert_result {
                    println!("Convert: {} -> {}", convert_result.before_extension.to_string(), convert_result.after_extension.to_string());
                }
                if let Some(trim_result) = thread_results.trim_result {
                    println!("Trim: {}x{} -> {}x{}", trim_result.before_size.width, trim_result.before_size.height, trim_result.after_size.width, trim_result.after_size.height);
                }
                if let Some(resize_result) = thread_results.resize_result {
                    println!("Resize: {}x{} -> {}x{}", resize_result.before_size.width, resize_result.before_size.height, resize_result.after_size.width, resize_result.after_size.height);
                }
                if let Some(grayscale_result) = thread_results.grayscale_result {
                    if grayscale_result.status {
                        println!("Grayscale: Done.");
                    }
                }
                if let Some(compress_result) = thread_results.compress_result {
                    if compress_result.status {
                        println!("Compress: Done.");
                    }
                }
                match thread_results.save_result.status {
                    RusimgStatus::Success => {
                        // 保存先などの表示
                        save_print(&thread_results.save_result.input_path, &thread_results.save_result.output_path,
                            thread_results.save_result.before_filesize, thread_results.save_result.after_filesize);

                        if thread_results.save_result.delete {
                            println!("Delete source file: {}", thread_results.save_result.input_path.display());
                        }
                        println!("{}", "Success.".green().bold())
                    },
                    RusimgStatus::Cancel => println!("{}", "Canceled.".yellow().bold()),
                    RusimgStatus::NotNeeded => {},
                };
            }
            Err(e) => {
                println!("{}: {}", "Error".red(), e.to_string());
                error_count = error_count + 1;
            }
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
