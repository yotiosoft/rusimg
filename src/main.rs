use std::path::{Path, PathBuf};
use std::fs;

mod parse;
mod rusimg;
use file_matcher::FilesNamed;

fn get_files_in_dir(dir_path: &String) -> Result<Vec<PathBuf>, String> {
    let mut files = fs::read_dir(&dir_path).expect("cannot read directory");
    let mut ret = Vec::new();

    while let Some(file) = files.next() {
        let dir_entry = file;
        match dir_entry {
            Ok(dir_entry) => {
                let path = dir_entry.path();
                if path.is_dir() {
                    let mut files = get_files_in_dir(&path.into_os_string().into_string().expect("cannot convert file name"))?;
                    ret.append(&mut files);
                }
                else {
                    let file_name = dir_entry.file_name().into_string().expect("cannot convert file name");
                    if rusimg::get_extension(&file_name).is_ok() {
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

fn get_files_by_regex(source_path_str: &String) -> Result<Vec<PathBuf>, String> {
    let path = PathBuf::from(source_path_str);
    let mut parent_path = path.parent();
    let child_path = path.file_name();
    if parent_path.is_none() || parent_path.unwrap().to_str().unwrap() == "" {
        parent_path = Some(Path::new("."));
    }
    if child_path.is_none() {
        return Err("cannot get file name".to_string());
    }

    let v = FilesNamed::wildmatch(child_path.unwrap().to_str().unwrap())
        .within(parent_path.unwrap())
        .find();

    if let Ok(v) = v {
        Ok(v)
    }
    else {
        Err("cannot get files by wildmatch".to_string())
    }
}

fn main() -> Result<(), String> {
    let args = parse::parser();

    let source_path = Path::new(&args.souce_path);
    let image_files = if source_path.is_dir() {
        get_files_in_dir(&args.souce_path)?
    }
    else {
        get_files_by_regex(&args.souce_path)?
    };

    for image_file_path in image_files {
        println!("[Processing: {}]", &Path::new(&image_file_path).file_name().unwrap().to_str().unwrap());

        // ファイルを開く
        let mut image = rusimg::open_image(&image_file_path).map_err(|e| e.to_string())?;

        // --trim -> トリミング
        if let Some(trim) = args.trim {
            // トリミング
            match rusimg::trim(&mut image, (trim.0.0, trim.0.1), (trim.1.0, trim.1.1)) {
                Ok(_) => (),
                Err(e) => return Err(e.to_string()),
            }
        }

        // --resize -> リサイズ
        if let Some(resize) = args.resize {
            // リサイズ
            match rusimg::resize(&mut image, resize) {
                Ok(_) => (),
                Err(e) => return Err(e.to_string()),
            }
        }

        // --convert -> 変換
        if let Some(ref c) = args.destination_extension {
            let extension = rusimg::get_extension(&c).map_err(|e| e.to_string())?;

            // 変換
            match rusimg::convert(&mut image, &extension) {
                Ok(img) => image = img,
                Err(e) => return Err(e.to_string()),
            }
        }

        // --grayscale -> グレースケール
        if args.grayscale {
            // グレースケール
            rusimg::grayscale(&mut image);
        }

        // --quality -> 圧縮
        if let Some(q) = args.quality {
            // 圧縮
            match rusimg::compress(&mut image.data, &image.extension, Some(q)) {
                Ok(_) => (),
                Err(e) => return Err(e.to_string()),
            }
        }

        // 出力
        let output_path = match &args.destination_path {
            Some(path) => Some(path),
            None => None,
        };
        let saved_filepath = rusimg::save_image(output_path, &mut image.data, &image.extension).map_err(|e| e.to_string())?;

        // --delete -> 元ファイルの削除 (optinal)
        if args.delete && image_file_path != saved_filepath {
            match fs::remove_file(&image_file_path) {
                Ok(_) => (),
                Err(e) => return Err(e.to_string()),
            }
        }

        // 表示
        if args.view {
            match rusimg::view(&mut image) {
                Ok(_) => (),
                Err(e) => return Err(e.to_string()),
            }
        }

        println!("Done.");
    }

    print!("All images are processed.");

    Ok(())
}
