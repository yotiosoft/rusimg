use std::path::PathBuf;
use clap::Parser;
use regex::Regex;

pub struct ArgStruct {
    pub souce_path: Option<PathBuf>,
    pub destination_path: Option<PathBuf>,
    pub destination_extension: Option<String>,
    pub quality: Option<f32>,
    pub delete: bool,
    pub resize: Option<u8>,
    pub trim: Option<((u32, u32), (u32, u32))>,
    pub grayscale: bool,
    pub view: bool,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source file path (file name or directory path)
    source: Option<PathBuf>,

    /// Destination file path (file name or directory path)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Destination file extension (e.g. jpeg, png, webp, bmp)
    #[arg(short, long)]
    convert: Option<String>,

    /// Resize images in parcent (must be 0 < resize <= 100)
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
}

pub fn parser() -> ArgStruct {
    let args = Args::parse();

    let trim = if args.trim.is_some() {
        let re = Regex::new(r"^\d*x\d*\+\d*\+\d*$").unwrap();
        let trim = args.trim.unwrap();
        if re.is_match(&trim) {
            let trim_wh = trim.split("+").collect::<Vec<&str>>();
            let trim_xy = trim_wh[0].split("x").collect::<Vec<&str>>();
            let x = trim_xy[0].parse::<u32>().unwrap();
            let y = trim_xy[1].parse::<u32>().unwrap();
            let w = trim_wh[1].parse::<u32>().unwrap();
            let h = trim_wh[2].parse::<u32>().unwrap();
            Some(((x, y), (w, h)))
        }
        else {
            println!("Invalid trim format. Please use 'XxY+W+H' (e.g.100x100+50x50).");
            std::process::exit(1);
        }
    }
    else {
        None
    };

    ArgStruct {
        souce_path: args.source,
        destination_path: args.output,
        destination_extension: args.convert,
        quality: args.quality,
        delete: args.delete,
        resize: args.resize,
        trim,
        grayscale: args.grayscale,
        view: args.view,
    }
}
