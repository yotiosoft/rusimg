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
                        ret.push(file_name);
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

    for image_file in image_files {
        println!("Processing {}...", &image_file);

        // ファイルを開く
        let mut image = rusimg::open_image(&image_file)?;

        // 各モードの処理
        match args.execution_mode {
            parse::ExecutionMode::Compress => {
                // 圧縮
                match rusimg::compress(&mut image.data, &image.extension) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
            },
            parse::ExecutionMode::Convert => {
                let extension = match args.destination_extension {
                    Some(ref extension) => rusimg::get_extension(&extension)?,
                    None => return Err("Destination extension is not specified".to_string()),
                };

                // 変換
                match rusimg::convert(&mut image.data, &image.extension, &extension) {
                    Ok(img) => image = img,
                    Err(e) => return Err(e),
                }
            },
            _ => (),
        }

        // 出力
        let output_path = match &args.destination_path {
            Some(path) => Some(path),
            None => None,
        };
        rusimg::save_image(output_path, &mut image.data, &image.extension)?;

        println!("Done.");
    }

    Ok(())
}
