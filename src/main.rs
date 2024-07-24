use std::path::{Path, PathBuf};
use std::fs;
use std::fmt::{self, Error};
use std::io::{stdout, Write};
use glob::glob;
use image::DynamicImage;
use parse::ArgStruct;
use colored::*;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use futures::stream::FuturesUnordered;

use rusimg::RusimgError;
mod parse;

// error type
type ErrorOccuredFilePath = String;
type ErrorMessage = std::io::Error;
struct ErrorStruct<T> {
    error: T,
    filepath: ErrorOccuredFilePath,
}
enum ProcessingError {
    RusimgError(ErrorStruct<RusimgError>),
    IOError(ErrorStruct<ErrorMessage>),
    ArgError(ErrorStruct<ErrorMessage>),
}
impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessingError::RusimgError(e) => write!(f, "{}", e.error),
            ProcessingError::IOError(e) => write!(f, "{}", e.error),
            ProcessingError::ArgError(e) => write!(f, "{}", e.error),
        }
    }
}

// result status
#[derive(Debug, Clone, PartialEq)]
enum FileOverwriteAsk {
    YesToAll,
    NoToAll,
    AskEverytime,
}
enum ExistsCheckResult {
    AllOverwrite,
    AllSkip,
    NeedToAsk,
    NoProblem,
}
enum AskResult {
    Overwrite,
    Skip,
    NoProblem,
}

#[derive(Debug, Clone, PartialEq)]
enum RusimgStatus {
    Success,
    Cancel,
    NotNeeded,
}

// thread task
struct ThreadTask {
    args: ArgStruct,
    input_path: PathBuf,
    output_path: Option<PathBuf>,
    extension: rusimg::Extension,
    ask_result: AskResult,
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
struct ProcessResult {
    viuer_image: Option<DynamicImage>,
    convert_result: Option<ConvertResult>,
    trim_result: Option<TrimResult>,
    resize_result: Option<ResizeResult>,
    grayscale_result: Option<GrayscaleResult>,
    compress_result: Option<CompressResult>,
    save_result: SaveResult,
}
struct ThreadResult {
    process_result: Option<Result<ProcessResult, ProcessingError>>,
    finish: bool,
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

// 保存先パスの決定
fn get_output_path(args: &ArgStruct, input_path: &PathBuf, extension: &rusimg::Extension) -> PathBuf {
    // 出力先パスを決定
    let mut output_path = match &args.destination_path {
        Some(path) => path.clone(),                                                             // If --output is specified, use it
        None => Path::new(input_path).with_extension(extension.to_string()),       // If not, use the input filepath as the input file
    };
    // append_name が指定されている場合、ファイル名に追加
    if let Some(append_name) = &args.destination_append_name {
        let mut output_path_tmp = output_path.file_stem().unwrap().to_str().unwrap().to_string();
        output_path_tmp.push_str(append_name);
        output_path_tmp.push_str(".");
        output_path_tmp.push_str(&extension.to_string());
        output_path = PathBuf::from(output_path_tmp);
    }
    output_path
}

// ファイルの存在チェック
fn check_file_exists(path: &PathBuf, file_overwrite_ask: &FileOverwriteAsk) -> ExistsCheckResult {
    // ファイルの存在チェック
    // ファイルが存在する場合、上書きするかどうかを確認
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

// ファイルを上書きするか確認する
fn ask_file_exists() -> bool {
    print!(" Do you want to overwrite it? [y/N]: ");
    loop {
        stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim().to_ascii_lowercase() == "y" || input.trim().to_ascii_lowercase() == "yes" {
            println!(" => The file will be overwritten.");
            return true;
        }
        else if input.trim().to_ascii_lowercase() == "n" || input.trim().to_ascii_lowercase() == "no" || input.trim() == "" {
            println!(" => The file will be skipped.");
            return false;
        }
        else {
            print!(" Please enter y or n [y/N]: ");
        }
    }
}

// 保存先などの表示
fn save_print(before_path: &PathBuf, after_path: &Option<PathBuf>, before_size: u64, after_size: Option<u64>) {
    match (after_path, after_size) {
        (Some(after_path), Some(after_size)) => {
            if before_path == after_path {
                println!("{}: {}", "Overwrite", before_path.display());
                println!("File Size: {} -> {} ({:.1}%)", before_size, after_size, (after_size as f64 / before_size as f64) * 100.0);
            }
            else if get_extension(before_path.as_path()) != get_extension(after_path.as_path()) {
                println!("{}: {} -> {}", "Rename", before_path.display(), after_path.display());
                println!("File Size: {} -> {} ({:.1}%)", before_size, after_size, (after_size as f64 / before_size as f64) * 100.0);
            }
            else {
                println!("{}: {} -> {}", "Move", before_path.display(), after_path.display());
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

// 各スレッドでの処理
async fn process(thread_task: ThreadTask, file_io_lock: Arc<Mutex<i32>>) -> Result<ProcessResult, ProcessingError> {
    let args = thread_task.args;
    let image_file_path = thread_task.input_path;
    let output_file_path = thread_task.output_path;
    let ask_result = thread_task.ask_result;

    let rierr = |e: RusimgError| ProcessingError::RusimgError(ErrorStruct { error: e, filepath: image_file_path.to_str().unwrap().to_string() });
    let ioerr = |e: std::io::Error| ProcessingError::IOError(ErrorStruct { error: e, filepath: image_file_path.to_str().unwrap().to_string() });

    // ファイルを開く
    let mut image = rusimg::open_image(&image_file_path).map_err(rierr)?;

    // 保存が必要か？
    let mut save_required = false;

    // --convert -> 画像形式変換
    let convert_result = if let Some(_c) = args.destination_extension {
        let extension = thread_task.extension;
        let before_extension = image.extension.clone();

        // 変換
        image.convert(&extension).map_err(rierr)?;
        save_required = true;

        Some(ConvertResult {
            before_extension: before_extension,
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
        let trimmed_size = image.trim_rect(trim).map_err(rierr)?;
        if before_size != trimmed_size {
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
        save_required = true;

        Some(CompressResult {
            status: true,
        })
    }
    else {
        None
    };

    // for viuer
    let viuer_image = if args.view {
        Some(image.get_dynamic_image().map_err(rierr)?)
    }
    else {
        None
    };

    // 出力
    let save_status = if save_required == true {
        // ファイルの存在チェック
        match ask_result {
            AskResult::Overwrite => {
                // そのまま保存へ
            },
            AskResult::Skip => {
                return Ok(ProcessResult {
                    viuer_image: viuer_image,
                    convert_result: convert_result,
                    trim_result: trim_result,
                    resize_result: resize_result,
                    grayscale_result: grayscale_result,
                    compress_result: compress_result,
                    save_result: SaveResult {
                        status: RusimgStatus::Cancel,
                        input_path: image.get_input_filepath(),
                        output_path: None,
                        before_filesize: 0,
                        after_filesize: None,
                        delete: false,
                    },
                });
            },
            AskResult::NoProblem => {
                // そのまま保存へ
            },
        }

        // 出力先パス
        let output_path = output_file_path.unwrap();

        // 保存
        // 出力処理は同時に実行すると負荷がかかるため、排他制御を行う
        let save_status = {
            let mut lock = file_io_lock.lock().unwrap();
            *lock += 1;
            let ret = image.save_image(output_path.to_str()).map_err(rierr)?;
            ret
        };

        // --delete -> 元ファイルの削除 (optinal)
        let delete = if let Some(saved_filepath) = save_status.output_path.clone() {
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

    let thread_results = ProcessResult {
        viuer_image: viuer_image,
        convert_result: convert_result,
        trim_result: trim_result,
        resize_result: resize_result,
        grayscale_result: grayscale_result,
        compress_result: compress_result,
        save_result: save_status,
    };

    Ok(thread_results)
}

#[tokio::main]
async fn main() -> Result<(), String> {
    // 引数のパース
    let args = parse::parser();

    // スレッド数
    let threads = args.threads;

    // 上書きが必要な場合、毎回確認するかどうか
    let file_overwrite_ask = if args.yes {
        FileOverwriteAsk::YesToAll
    }
    else if args.no {
        FileOverwriteAsk::NoToAll
    }
    else {
        FileOverwriteAsk::AskEverytime
    };

    // 作業ディレクトリの指定（default: current dir）
    let source_paths = args.souce_path.clone().or(Some(vec![PathBuf::from(".")])).unwrap();
    let mut thread_tasks = Vec::new();
    for source_path in source_paths {
        let image_files_temp = if source_path.is_dir() {
            get_files_in_dir(&source_path, args.recursive)?
        }
        else {
            get_files_by_wildcard(&source_path)?
        };
        for image_file in image_files_temp {
            // 出力先パスを決定
            let extension = convert_str_to_extension(&args.destination_extension.clone().unwrap_or("".to_string())).unwrap();
            let output_path = get_output_path(&args, &image_file, &extension);

            // 出力先が既に存在する場合、上書きするかどうかを確認
            let ask_result = match check_file_exists(&output_path, &file_overwrite_ask) {
                ExistsCheckResult::AllOverwrite => {
                    println!("{}", " => Overwrite (default: yes)".bold());
                    AskResult::Overwrite
                },
                ExistsCheckResult::AllSkip => {
                    println!("{}", " => Skip (default: no)".bold());
                    AskResult::Skip
                },
                ExistsCheckResult::NeedToAsk => {
                    if ask_file_exists() {
                        AskResult::Overwrite
                    }
                    else {
                        AskResult::Skip
                    }
                },
                ExistsCheckResult::NoProblem => {
                    AskResult::NoProblem
                },
            };

            let thread_task = ThreadTask {
                args: args.clone(),
                input_path: image_file,
                output_path: Some(output_path),
                extension: extension,
                ask_result: ask_result,
            };

            thread_tasks.push(thread_task);
        }
    }

    // 検出した画像ファイルパスの表示
    let total_image_count = thread_tasks.len();
    println!("{}", format!("🔎 {} images are detected.", total_image_count).bold());

    // thread_tasks をスレッド間で共有
    let thread_tasks = Arc::new(Mutex::new(thread_tasks));

    // 各画像に対する処理
    let mut error_count = 0;
    let count = Arc::new(Mutex::new(0));
    let tasks = FuturesUnordered::new();
    
    // チャネルを用意
    let (tx, mut rx) = mpsc::channel::<ThreadResult>(32);

    let file_io_lock = Arc::new(Mutex::new(0));

    for _thread_num in 0..threads {
        let thread_tasks = Arc::clone(&thread_tasks);
        let count = Arc::clone(&count);
        let tx = tx.clone();
        let file_io_lock = Arc::clone(&file_io_lock);
        
        let thread = tokio::spawn(async move {
            loop {
                let thread_task = {
                    let mut thread_tasks = thread_tasks.lock().unwrap();
                    thread_tasks.pop()
                };
                if thread_task.is_none() {
                    match tx.send(ThreadResult {
                        process_result: None,
                        finish: true,
                    }).await {
                        Ok(_) => {},
                        Err(e) => {
                            println!("Send error: {}", e.to_string());
                        }
                    }
                    break;
                }
                let thread_task = thread_task.unwrap();
                /*
                let processing_str = format!("[{}/{}] Processing: {}", count, total_image_count, &Path::new(&thread_task.input_path).file_name().unwrap().to_str().unwrap());
                println!("{}", processing_str.yellow().bold());
                */
                let process_result = process(thread_task, file_io_lock.clone()).await;
                match tx.send(ThreadResult {
                    process_result: Some(process_result),
                    finish: false,
                }).await {
                    Ok(_) => {},
                    Err(e) => {
                        println!("Send error: {}", e.to_string());
                    }
                }

                // カウント
                let mut count = count.lock().unwrap();
                *count += 1;
            }
        });
        tasks.push(thread);
    }

    // スレッドの実行結果を表示
    let mut count = 0;
    let mut thread_finished = 0;
    while let Some(rx_result) = rx.recv().await {
        if let Some(process_result) = rx_result.process_result {
            match process_result {
                Ok(thread_results) => {
                    count = count + 1;
                    let processing_str = format!("[{}/{}] Finish: {}", count + error_count, total_image_count, &Path::new(&thread_results.save_result.input_path).file_name().unwrap().to_str().unwrap());
                    println!("{}", processing_str.yellow().bold());

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

                    // 表示 (viuer)
                    if let Some(viuer_image) = thread_results.viuer_image {
                        view(&viuer_image).map_err(|e| e.to_string()).unwrap();
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
                        RusimgStatus::NotNeeded => println!("{}", "Nothing to do.".yellow().bold()),
                    };
                }
                Err(e) => {
                    error_count = error_count + 1;
                    match e {
                        ProcessingError::RusimgError(e) => {
                            let processing_str = format!("[{}/{}] Failed: {}", count + error_count, total_image_count, e.filepath);
                            println!("{}", processing_str.red().bold());
                            println!("{}: {}", "Error".red(), e.error);
                        },
                        ProcessingError::IOError(e) => {
                            let processing_str = format!("[{}/{}] Failed: {}", count + error_count, total_image_count, e.filepath);
                            println!("{}", processing_str.red().bold());
                            println!("{}: {}", "Error".red(), e.error);
                        },
                        ProcessingError::ArgError(e) => {
                            let processing_str = format!("[{}/{}] Failed: {}", count + error_count, total_image_count, e.filepath);
                            println!("{}", processing_str.red().bold());
                            println!("{}: {}", "Error".red(), e.error);
                        },
                    }
                }
            }
        }

        if rx_result.finish {
            thread_finished = thread_finished + 1;
        }
        if thread_finished == threads {
            break;
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
