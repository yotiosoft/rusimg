use std::path::PathBuf;
use clap::Parser;
use regex::Regex;
use librusimg::Rect;
use std::fmt;

const DEFAULT_THREADS: u8 = 4;

/// Argument errors
#[derive(Debug)]
pub enum ArgError {
    InvalidTrimFormat,
    FailedToParseTrim(String),
    InvalidQuality,
    InvalidResize,
    InvalidThreads,
}
impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgError::InvalidTrimFormat => write!(f, "Invalid trim format. Please use 'XxY+W+H' (e.g.100x100+50x50)."),
            ArgError::FailedToParseTrim(e) => write!(f, "Failed to parse trim format: \n\t{}", e),
            ArgError::InvalidQuality => write!(f, "Quality must be 0.0 <= q <= 100.0"),
            ArgError::InvalidResize => write!(f, "Resize must be size > 0"),
            ArgError::InvalidThreads => write!(f, "Threads must be threads => 1"),
        }
    }

}

/// Argument structure
/// souce_path: Option<Vec<PathBuf>>: Source file path (file name or directory path)
/// destination_path: Option<PathBuf>: Destination file path (file name or directory path)
/// destination_extension: Option<String>: Destination file extension (e.g. jpeg, png, webp, bmp)
/// destination_append_name: Option<String>: Name to be appended to the source file name (e.g. image.jpg -> image_output.jpg)
/// recursive: bool: Recusive search (default: false)
/// quality: Option<f32>: Image quality (for compress, must be 0.0 <= q <= 100.0)
/// delete: bool: Delete source file (default: false)
/// resize: Option<u8>: Resize images in parcent (must be 0 < size)
/// trim: Option<Rect>: Trim image. trim: librusimg::Rect { x: u32, y: u32, w: u32, h: u32 }
/// grayscale: bool: Grayscale image (default: false)
/// view: bool: View result in the comand line (default: false)
/// yes: bool: Yes to all (default: false) to overwrite files
/// no: bool: No to all (default: false) to overwrite files
/// threads: u8: Number of threads (default: 4)
#[derive(Debug, Clone)]
pub struct ArgStruct {
    pub souce_path: Option<Vec<PathBuf>>,
    pub destination_path: Option<PathBuf>,
    pub destination_extension: Option<String>,
    pub destination_append_name: Option<String>,
    pub recursive: bool,
    pub quality: Option<f32>,
    pub delete: bool,
    pub resize: Option<f32>,
    pub trim: Option<Rect>,
    pub grayscale: bool,
    pub view: bool,
    pub yes: bool,
    pub no: bool,
    pub double_extension: bool,
    pub threads: u8,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Source file path (file name or directory path).  
    /// If not specified, the current directory will be used.  
    /// This option can be used multiple times.
    #[arg(short, long, num_args(0..))]
    input: Option<Vec<PathBuf>>,
    
    /// Recursively process all files in the directory.
    #[arg(long)]
    recursive: bool,

    /// Specify output directory or output file name. 
    /// If not specified, the input file name will be used.  
    /// This option can be used only once.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Name to be appended to the source file name
    /// (e.g. image.jpg -> image_output.jpg)
    #[arg(short, long)]
    append: Option<String>,

    /// Destination file extension (e.g. jpeg, png, webp, bmp).
    #[arg(short, long)]
    convert: Option<String>,

    /// Resize images in parcent (must be 0 < size)
    #[arg(short, long)]
    resize: Option<f32>,

    /// Trim image. Input format: 'XxY+W+H' (e.g.100x100+50x50)
    #[arg(short, long)]
    trim: Option<String>,

    /// Grayscale image
    #[arg(short, long)]
    grayscale: bool,

    /// Image quality (for compress, must be 0.0 <= q <= 100.0)
    #[arg(short, long)]
    quality: Option<f32>,

    /// Set output file extension to double extension (e.g. image.jpg -> image.jpg.webp)
    #[arg(short, long)]
    double_extension: bool,

    /// View result in the comand line
    #[arg(short, long)]
    view: bool,

    /// Yes to all to overwrite files
    #[arg(short, long)]
    yes: bool,

    /// No to all to overwrite files
    #[arg(short, long)]
    no: bool,

    /// Delete source file
    #[arg(short='D', long)]
    delete: bool,

    /// Number of threads.
    #[arg(short='T', long, default_value_t = DEFAULT_THREADS)]
    threads: u8,
}

pub fn check_trim_format(trim: &str) -> Result<Rect, String> {
    let re = Regex::new(r"(\d+)x(\d+)\+(\d+)x(\d+)").unwrap();
    if let Some(captures) = re.captures(trim) {
        let x = captures.get(1).unwrap().as_str().parse().map_err(|e: std::num::ParseIntError| e.to_string()).unwrap();
        let y = captures.get(2).unwrap().as_str().parse().map_err(|e: std::num::ParseIntError| e.to_string()).unwrap();
        let w = captures.get(3).unwrap().as_str().parse().map_err(|e: std::num::ParseIntError| e.to_string()).unwrap();
        let h = captures.get(4).unwrap().as_str().parse().map_err(|e: std::num::ParseIntError| e.to_string()).unwrap();
        Ok(Rect{x, y, w, h})
    }
    else {
        Err("Invalid trim format".to_string())
    }
}

pub fn check_quality_range(quality: Option<f32>) -> bool {
    if let Some(q) = quality {
        if q < 0.0 || q > 100.0 {
            return false;
        }
    }
    true
}

pub fn check_resize_range(resize: Option<f32>) -> bool {
    if let Some(r) = resize {
        if r <= 0.0 {
            return false;
        }
    }
    true
}

pub fn check_threads_range(threads: u8) -> bool {
    if threads < 1 {
        return false;
    }
    true
}

fn check_and_generate(args: Args) -> Result<ArgStruct, ArgError> {
    // If trim option is specified, check the format.
    let trim: Result<Option<librusimg::Rect>, String> = if args.trim.is_some() {
        let trim = check_trim_format(args.trim.as_ref().unwrap());
        if trim.is_err() {
            return Err(ArgError::InvalidTrimFormat);
        }
        else {
            let trim = trim.unwrap();
            Ok(Some(trim))
        }
    }
    else {
        Ok(None)
    };
    let trim = if let Err(e) = trim {
        return Err(ArgError::FailedToParseTrim(e));
    }
    else {
        trim.unwrap()
    };

    if args.quality.is_some() && !check_quality_range(args.quality) {
        return Err(ArgError::InvalidQuality);
    }

    if args.resize.is_some() && !check_resize_range(args.resize) {
        return Err(ArgError::InvalidResize);
    }

    if !check_threads_range(args.threads) {
        return Err(ArgError::InvalidThreads);
    }

    Ok(ArgStruct {
        souce_path: args.input,
        destination_path: args.output,
        destination_extension: args.convert,
        destination_append_name: args.append,
        recursive: args.recursive,
        quality: args.quality,
        delete: args.delete,
        resize: args.resize,
        trim,
        grayscale: args.grayscale,
        view: args.view,
        yes: args.yes,
        no: args.no,
        double_extension: args.double_extension,
        threads: args.threads,
    })
}

pub fn parser() -> Result<ArgStruct, ArgError> {
    // Parse arguments.
    let args = Args::parse();
    // Check and generate arguments.
    let args = check_and_generate(args)?;
    Ok(args)
}
