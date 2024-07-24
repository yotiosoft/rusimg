use std::path::PathBuf;
use clap::Parser;
use regex::Regex;
use rusimg::Rect;
use std::fmt;

const DEFAULT_THREADS: usize = 4;

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
            ArgError::FailedToParseTrim(e) => write!(f, "Failed to parse trim format: {}", e),
            ArgError::InvalidQuality => write!(f, "Quality must be 0.0 <= q <= 100.0"),
            ArgError::InvalidResize => write!(f, "Resize must be size > 0"),
            ArgError::InvalidThreads => write!(f, "Threads must be threads => 1"),
        }
    }

}

#[derive(Debug, Clone)]
pub struct ArgStruct {
    pub souce_path: Option<Vec<PathBuf>>,
    pub destination_path: Option<PathBuf>,
    pub destination_extension: Option<String>,
    pub destination_append_name: Option<String>,
    pub recursive: bool,
    pub quality: Option<f32>,
    pub delete: bool,
    pub resize: Option<u8>,
    pub trim: Option<Rect>,
    pub grayscale: bool,
    pub view: bool,
    pub yes: bool,
    pub no: bool,
    pub threads: usize,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source file path (file name or directory path)
    source: Option<Vec<PathBuf>>,

    /// Recusive search
    #[arg(long)]
    recursive: bool,

    /// Destination file path (file name or directory path)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Name to be appended to the source file name
    /// (e.g. image.jpg -> image_output.jpg)
    #[arg(short, long)]
    append: Option<String>,

    /// Destination file extension (e.g. jpeg, png, webp, bmp)
    #[arg(short, long)]
    convert: Option<String>,

    /// Resize images in parcent (must be 0 < size)
    #[arg(short, long)]
    resize: Option<u8>,

    /// Trim image
    #[arg(short, long)]
    trim: Option<String>,

    /// Grayscale image
    #[arg(short, long)]
    grayscale: bool,

    /// Image quality (for compress, must be 0.0 <= q <= 100.0)
    #[arg(short, long)]
    quality: Option<f32>,

    /// Delete source file
    #[arg(short, long)]
    delete: bool,

    /// View result in the comand line
    #[arg(short, long)]
    view: bool,

    /// Yes to all
    #[arg(short, long)]
    yes: bool,

    /// Number of threads
    #[arg(short='T', long, default_value_t = DEFAULT_THREADS)]
    threads: usize,

    /// No to all
    #[arg(short, long)]
    no: bool,
}

pub fn parser() -> Result<ArgStruct, ArgError> {
    // 引数のパース
    let args = Args::parse();

    // If trim option is specified, check the format.
    let trim: Result<Option<rusimg::Rect>, String> = if args.trim.is_some() {
        let re = Regex::new(r"(\d+)x(\d+)\+(\d+)x(\d+)").unwrap();
        if let Some(captures) = re.captures(&args.trim.unwrap()) {
            let x = captures.get(1).unwrap().as_str().parse().map_err(|e: std::num::ParseIntError| e.to_string()).unwrap();
            let y = captures.get(2).unwrap().as_str().parse().map_err(|e: std::num::ParseIntError| e.to_string()).unwrap();
            let w = captures.get(3).unwrap().as_str().parse().map_err(|e: std::num::ParseIntError| e.to_string()).unwrap();
            let h = captures.get(4).unwrap().as_str().parse().map_err(|e: std::num::ParseIntError| e.to_string()).unwrap();
            Ok(Some(Rect{x, y, w, h}))
        }
        else {
            return Err(ArgError::InvalidTrimFormat);
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

    if (args.quality < Some(0.0) || args.quality > Some(100.0)) && args.quality.is_some() {
        return Err(ArgError::InvalidQuality);
    }
    if args.resize < Some(0) && args.resize.is_some() {
        return Err(ArgError::InvalidResize);
    }

    if args.threads < 1 {
        return Err(ArgError::InvalidThreads);
    }

    Ok(ArgStruct {
        souce_path: args.source,
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
        threads: args.threads,
    })
}
