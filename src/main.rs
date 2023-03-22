use std::path::Path;
use std::fs;

mod parse;
mod rusimg;

fn get_files_in_dir(dir_path: &String) -> Result<Vec<String>, String> {
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
                        ret.push(Path::new(&dir_path).join(&file_name).to_str().expect("cannot convert file name").to_string());
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

fn main() -> Result<(), String> {
    let args = parse::parser();

    let source_path = Path::new(&args.souce_path);
    let image_files = if source_path.is_dir() {
        get_files_in_dir(&args.souce_path)?
    }
    else {
        vec![args.souce_path]
    };

    for image_file_path in image_files {
        println!("[Processing: {}]", &Path::new(&image_file_path).file_name().unwrap().to_str().unwrap());

        // ファイルを開く
        let mut image = rusimg::open_image(&image_file_path)?;

        // --resize -> リサイズ
        if let Some(resize) = args.resize {
            // リサイズ
            match rusimg::resize(&mut image, resize) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }

        // --convert -> 変換
        if let Some(ref c) = args.destination_extension {
            let extension = rusimg::get_extension(&c)?;

            // 変換
            match rusimg::convert(&mut image, &extension) {
                Ok(img) => image = img,
                Err(e) => return Err(e),
            }
        }

        // --quality -> 圧縮
        if let Some(q) = args.quality {
            // 圧縮
            match rusimg::compress(&mut image.data, &image.extension, Some(q)) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }

        // 出力
        let output_path = match &args.destination_path {
            Some(path) => Some(path),
            None => None,
        };
        let saved_filepath = rusimg::save_image(output_path, &mut image.data, &image.extension)?;

        // 元ファイルの削除 (optinal: -d)
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
                Err(e) => return Err(e),
            }
        }

        println!("Done.");
    }

    Ok(())
}
