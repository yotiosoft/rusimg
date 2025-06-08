use std::path::{Path, PathBuf};
use std::fs;
use std::fmt;
use std::io::{stdout, Write};
use glob::glob;
use image::DynamicImage;
use colored::*;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use futures::stream::FuturesUnordered;

use librusimg::{RusImg, RusimgError};
mod background;

/// ThreadTask is a structure that represents the task to be executed by each thread.
/// - args: Arguments passed to the program.
/// - input_path: The path to the input image file.
/// - output_path: The path to the output image file.
/// - extension: The extension of the output image file.
/// - ask_result: The result of asking whether to overwrite the file.
pub struct ThreadTask {
    args: background::parse::ArgStruct,
    input_path: PathBuf,
    output_path: Option<PathBuf>,
    extension: Option<librusimg::Extension>,
    ask_result: AskResult,
}

/// AskResult is an enum that represents the result of asking whether to overwrite a file.
/// - Overwrite: Overwrite the file.
/// - Skip: Skip the file.
/// - NoProblem: No problem. This means that the file does not exist.
pub enum AskResult {
    Overwrite,
    Skip,
    NoProblem,
}

/// ProcessResult is a structure that represents the result of processing an image.
/// This structure contains the results of each processing step.
struct ProcessResult {
    viuer_image: Option<DynamicImage>,
    convert_result: Option<background::ConvertResult>,
    trim_result: Option<background::TrimResult>,
    resize_result: Option<background::ResizeResult>,
    grayscale_result: Option<background::GrayscaleResult>,
    compress_result: Option<background::CompressResult>,
    save_result: background::SaveResult,
}
/// ThreadResult is a structure that represents the result of processing an image in a thread.
/// This structure contains the processing result and a flag indicating whether the processing is complete.
struct ThreadResult {
    process_result: Option<Result<ProcessResult, background::ProcessingError>>,
    finish: bool,
}

/// Ask if the file should be overwritten.
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

/// Show the result of saving the image.
fn save_print(before_path: &PathBuf, after_path: &Option<PathBuf>, before_size: u64, after_size: Option<u64>) {
    match (after_path, after_size) {
        (Some(after_path), Some(after_size)) => {
            if before_path == after_path {
                println!("{}: {}", "Overwrite", before_path.display());
                println!("File Size: {} -> {} ({:.1}%)", before_size, after_size, (after_size as f64 / before_size as f64) * 100.0);
            }
            else if background::get_extension(before_path.as_path()) != background::get_extension(after_path.as_path()) {
                println!("{}: {}", "Create", after_path.display());
                println!("File Size: {} -> {} ({:.1}%)", before_size, after_size, (after_size as f64 / before_size as f64) * 100.0);
            }
            else {
                println!("{}: {} -> {}", "Copy", before_path.display(), after_path.display());
                println!("File Size: {} -> {} ({:.1}%)", before_size, after_size, (after_size as f64 / before_size as f64) * 100.0);
            }
        },
        (_, _) => {
            return;
        },
    }
}

/// Process the image in a thread.
async fn process(thread_task: ThreadTask, file_io_lock: Arc<Mutex<i32>>) -> Result<ProcessResult, background::ProcessingError> {
    let args = thread_task.args;
    let image_file_path = thread_task.input_path;
    let output_file_path = thread_task.output_path;
    let ask_result = thread_task.ask_result;

    let rierr = |e: RusimgError| background::ProcessingError::RusimgError(background::ErrorStruct { error: e, filepath: image_file_path.to_str().unwrap().to_string() });
    let ioerr = |e: std::io::Error| background::ProcessingError::IOError(background::ErrorStruct { error: e, filepath: image_file_path.to_str().unwrap().to_string() });

    // Open the image
    let mut image = librusimg::RusImg::open(&image_file_path).map_err(rierr)?;

    // Is saving the image required? (default: false)
    let mut save_required = false;

    // --convert -> Convert the image.
    let convert_result = if let Some(_c) = args.destination_extension {
        save_required = true;
        background::process_convert(&thread_task.extension, &mut image, rierr)?
    }
    else {
        None
    };

    // --trim -> Trim the image.
    let trim_result = if let Some(trim) = args.trim {
        save_required = true;
        background::process_trim(&mut image, trim, rierr)?
    }
    else {
        None
    };

    // --resize -> Resize the image.
    let resize_result = if let Some(resize) = args.resize {
        save_required = true;
        background::process_resize(&mut image, resize, rierr)?
    }
    else {
        None
    };

    // --grayscale -> Convert the image to grayscale.
    let grayscale_result = if args.grayscale {
        save_required = true;
        background::process_grayscale(&mut image, rierr)?
    }
    else {
        None
    };

    // --quality -> Compress the image.
    let compress_result = if let Some(q) = args.quality {
        save_required = true;
        background::process_compress(&mut image, Some(q), rierr)?
    }
    else {
        None
    };

    // --view -> View the image in the terminal.
    // Viuer will be called after all processing is complete.
    // So, store the image data in memory.
    let viuer_image = if args.view {
        Some(image.get_dynamic_image().map_err(rierr)?)
    }
    else {
        None
    };

    // Move or copy the image to the output path.
    // If the output path is not specified, the image will be saved in the same directory as the input file.
    if !save_required && output_file_path.is_some() && image_file_path != output_file_path.clone().unwrap() {
        save_required = true;
    }

    // Save the image if necessary.
    let save_status = if save_required == true {
        // Check if the file exists and ask if it should be overwritten.
        match ask_result {
            AskResult::Overwrite => {
                // If AskResult::Overwrite, overwrite the file without asking.
                // So we don't need to check if the file exists.
            },
            AskResult::Skip => {
                // If AskResult::Skip, skip the file.
                return Ok(ProcessResult {
                    viuer_image: viuer_image,
                    convert_result: convert_result,
                    trim_result: trim_result,
                    resize_result: resize_result,
                    grayscale_result: grayscale_result,
                    compress_result: compress_result,
                    save_result: SaveResult {
                        status: RusimgStatus::Cancel,
                        input_path: image.get_input_filepath().map_err(rierr)?,
                        output_path: None,
                        before_filesize: 0,
                        after_filesize: None,
                        delete: false,
                    },
                });
            },
            AskResult::NoProblem => {
                // If no problem, save the file.
            },
        }

        // Get the output path
        let output_path = output_file_path.unwrap();

        // Save the image
        // Saving images at the same time can be a heavy load, so we need to lock the file I/O.
        // *lock is used to lock the file I/O.
        let save_status = {
            let mut lock = file_io_lock.lock().unwrap();
            *lock += 1;
            let ret = image.save_image(output_path.to_str()).map_err(rierr)?;
            ret
        };

        // --delete -> Delete the original file. 
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

        // Return the result of saving the image.
        SaveResult {
            status: RusimgStatus::Success,
            input_path: image.get_input_filepath().map_err(rierr)?,
            output_path: save_status.output_path,
            before_filesize: save_status.before_filesize.unwrap_or(0),
            after_filesize: save_status.after_filesize,
            delete: delete,
        }
    }
    else {
        // If saving is not required, return the status as NotNeeded.
        SaveResult {
            status: RusimgStatus::NotNeeded,
            input_path: image.get_input_filepath().map_err(rierr)?,
            output_path: None,
            before_filesize: 0,
            after_filesize: None,
            delete: false,
        }
    };

    // Return the processing result.
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
    // Parse the arguments.
    let args = parse::parser().map_err(|e| e.to_string())?;

    // Number of threads.
    let threads = args.threads;

    // Is it necessary to confirm every time if overwriting is required?
    // -y, --yes: Always overwrite
    // -n, --no: Always skip
    // If neither is specified, ask every time.
    let file_overwrite_ask = if args.yes {
        FileOverwriteAsk::YesToAll
    }
    else if args.no {
        FileOverwriteAsk::NoToAll
    }
    else {
        FileOverwriteAsk::AskEverytime
    };

    // Specify the source path.
    // Default: current directory
    let source_paths = args.souce_path.clone().or(Some(vec![PathBuf::from(".")])).unwrap();
    let mut thread_tasks = Vec::new();
    for source_path in source_paths {
        let image_files_list = if source_path.is_dir() {
            get_files_in_dir(&source_path, args.recursive)?
        }
        else {
            get_files_by_wildcard(&source_path)?
        };
        for image_filepath in image_files_list {
            let thread_task = if is_save_required(&args) {
                // Determine the output path.
                let arg_dest_extension = if let Some(ext) = &args.destination_extension {
                    Some(convert_str_to_extension(ext).map_err(|e| e.to_string())?)
                }
                else {
                    None
                };
                let extension = get_destination_extension(&image_filepath, &arg_dest_extension);
                let output_path = get_output_path(&image_filepath, &args.destination_path, args.double_extension, &args.destination_append_name, &extension);

                // If the output file already exists, check if it should be overwritten.
                let ask_result = match check_file_exists(&output_path, &file_overwrite_ask) {
                    // Print the result of checking if the file exists.
                    ExistsCheckResult::AllOverwrite => {
                        println!("{}", " => Overwrite (default: yes)".bold());
                        AskResult::Overwrite
                    },
                    ExistsCheckResult::AllSkip => {
                        println!("{}", " => Skip (default: no)".bold());
                        AskResult::Skip
                    },
                    ExistsCheckResult::NeedToAsk => {
                        // If the file exists, ask if it should be overwritten.
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

                // Make a thread task.
                ThreadTask {
                    args: args.clone(),
                    input_path: image_filepath,
                    output_path: Some(output_path),
                    extension: Some(extension),
                    ask_result: ask_result,
                }
            }
            else {
                // If saving is not required, create a thread task without an output path.
                ThreadTask {
                    args: args.clone(),
                    input_path: image_filepath,
                    output_path: None,
                    extension: None,
                    ask_result: AskResult::NoProblem,
                }
            };
            
            // Add the thread task to the thread_tasks.
            thread_tasks.push(thread_task);
        }
    }

    // Display the number of images detected.
    let total_image_count = thread_tasks.len();
    println!("{}", format!("🔎 {} images are detected.", total_image_count).bold());

    // Share thread_tasks between threads.
    let thread_tasks = Arc::new(Mutex::new(thread_tasks));

    // Processing for each image..
    let mut error_count = 0;
    let count = Arc::new(Mutex::new(0));
    let tasks = FuturesUnordered::new();
    
    // Prepare a channel to communicate between threads.
    let (tx, mut rx) = mpsc::channel::<ThreadResult>(32);

    // Lock for file I/O
    let file_io_lock = Arc::new(Mutex::new(0));

    // Start processing in each thread.
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

                // Count up the number of processed images.
                let mut count = count.lock().unwrap();
                *count += 1;
            }
        });
        tasks.push(thread);
    }

    // Display the results of the threads.
    let mut count = 0;
    let mut thread_finished = 0;
    while let Some(rx_result) = rx.recv().await {
        if let Some(process_result) = rx_result.process_result {
            match process_result {
                // If the processing is successful, display the result.
                Ok(thread_results) => {
                    count = count + 1;
                    let processing_str = format!("[{}/{}] Finish: {}", count + error_count, total_image_count, &thread_results.save_result.input_path.display().to_string());
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

                    // Show the image in the terminal.
                    // Use viuer crate to display the image.
                    if let Some(viuer_image) = thread_results.viuer_image {
                        view(&viuer_image).map_err(|e| e.to_string()).unwrap();
                    }

                    match thread_results.save_result.status {
                        RusimgStatus::Success => {
                            // Print the result of saving the image.
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
                // If an error occurs during processing, display the error.
                Err(e) => {
                    error_count = error_count + 1;
                    match e {
                        ProcessingError::RusimgError(e) => {
                            let processing_str = format!("[{}/{}] Failed: {}", count + error_count, total_image_count, &e.filepath);
                            println!("{}", processing_str.red().bold());
                            println!("{}: {}", "Error".red(), e.error);
                        },
                        ProcessingError::IOError(e) => {
                            let processing_str = format!("[{}/{}] Failed: {}", count + error_count, total_image_count, &e.filepath);
                            println!("{}", processing_str.red().bold());
                            println!("{}: {}", "Error".red(), e.error);
                        },
                        ProcessingError::FailedToViewImage(s) => {
                            println!("{}: {}", "Error".red(), s);
                        },
                        ProcessingError::FailedToConvertExtension(e) => {
                            let processing_str = format!("[{}/{}] Failed: {}", count + error_count, total_image_count, &e.filepath);
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
        // If all threads are finished, break the loop.
        if thread_finished == threads {
            break;
        }
    }

    // Show the result of processing all images.
    if error_count > 0 {
        println!("\n✅ {} images are processed.", total_image_count - error_count);
        println!("❌ {} images are failed to process.", error_count);
    }
    else {
        println!("\n✅ All images are processed.");
    }

    Ok(())
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
