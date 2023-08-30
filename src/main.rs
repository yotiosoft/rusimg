use std::path::{Path, PathBuf};
use std::fs;
use std::fmt;
use std::io::Read;

mod parse;
mod rusimg;
use glob::glob;
use parse::ArgStruct;
use rusimg::RusimgError;

#[macro_export]
macro_rules! printlnc {
    () => {
        $crate::print!("  \n")
    };
    ($($arg:tt)*) => {{
        $crate::io::_print("  " + $crate::format_args_nl!($($arg)*));
    }};
}

pub enum ProcessingError {
    RusimgError(RusimgError),
    IOError(String),
}
impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessingError::RusimgError(e) => write!(f, "{}", e.to_string()),
            ProcessingError::IOError(e) => write!(f, "{}", e),
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
                    if rusimg::get_extension(&Path::new(&file_name)).is_ok() {
                        ret.push(Path::new(&dir_path).join(&file_name));
                    }
                }
            },
            Err(e) => {
                printlnc!("cannot read a directory entry: {}", e.to_string());
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
                if rusimg::get_extension(&path).is_ok() {
                    ret.push(path);
                }
            },
            Err(e) => printlnc!("{:?}", e),
        }
    }
    Ok(ret)
}

fn process(args: &ArgStruct, image_file_path: &PathBuf) -> Result<(), ProcessingError> {
    let rierr = |e: RusimgError| ProcessingError::RusimgError(e);
    let ioerr = |e: std::io::Error| ProcessingError::IOError(e.to_string());

    // ファイルを開く
    let mut image = rusimg::open_image(&image_file_path).map_err(rierr)?;

    // --convert -> 画像形式変換
    if let Some(ref c) = args.destination_extension {
        let extension = rusimg::convert_str_to_extension(&c).map_err(rierr)?;

        // 変換
        image = rusimg::convert(&mut image, &extension).map_err(rierr)?;
    }

    // --trim -> トリミング
    if let Some(trim) = args.trim {
        // トリミング
        rusimg::trim(&mut image, (trim.0.0, trim.0.1), (trim.1.0, trim.1.1)).map_err(rierr)?;
    }

    // --resize -> リサイズ
    if let Some(resize) = args.resize {
        // リサイズ
        rusimg::resize(&mut image, resize).map_err(rierr)?;
    }

    // --grayscale -> グレースケール
    if args.grayscale {
        // グレースケール
        rusimg::grayscale(&mut image).map_err(rierr)?;
    }

    // --quality -> 圧縮
    if let Some(q) = args.quality {
        // 圧縮
        rusimg::compress(&mut image.data, &image.extension, Some(q)).map_err(rierr)?;
    }

    // 出力
    let output_path = match &args.destination_path {
        Some(path) => Some(path),
        None => None,
    };
    let saved_filepath = rusimg::save_image(output_path, &mut image.data, &image.extension).map_err(rierr)?;

    // --delete -> 元ファイルの削除 (optinal)
    if args.delete && image_file_path != &saved_filepath {
        fs::remove_file(&image_file_path).map_err(ioerr)?;
    }

    // 表示
    if args.view {
        rusimg::view(&mut image).map_err(rierr)?;
    }

    Ok(())
}

fn main() -> Result<(), String> {
    // 引数のパース
    let args = parse::parser();

    // 作業ディレクトリの指定（default: current dir）
    let source_path = args.souce_path.clone().or(Some(PathBuf::from("."))).unwrap();
    let image_files = if source_path.is_dir() {
        get_files_in_dir(&source_path, args.recursive)?
    }
    else {
        get_files_by_wildcard(&source_path)?
    };

    // 検出した画像ファイルパスの表示
    println!("🔎 {} images are detected.", image_files.len());
    for image_file_path in &image_files {
        println!("  {}", image_file_path.to_str().unwrap());
    }
    printlnc!();

    // 各画像に対する処理
    for image_file_path in image_files {
        printlnc!(" [Processing: {}]", &Path::new(&image_file_path).file_name().unwrap().to_str().unwrap());

        match process(&args, &image_file_path) {
            Ok(_) => {},
            Err(e) => {
                printlnc!("Error: {}", e.to_string());
                continue;
            },
        }

        printlnc!("Done.");
    }

    println!("✅ All images are processed.");

    Ok(())
}
