mod parse;
mod rusimg;

fn main() -> Result<(), String> {
    let args = parse::parser();
    let mut image = rusimg::open_image(&args.souce_path)?;

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
                Some(extension) => rusimg::get_extension(&extension)?,
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
    let output_path = match args.destination_path {
        Some(path) => Some(path),
        None => None,
    };
    rusimg::save_image(&output_path, &mut image.data, &image.extension)?;

    Ok(())
}
