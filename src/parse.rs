use clap::Parser;
use regex::Regex;

pub struct ArgStruct {
    pub souce_path: String,
    pub destination_path: Option<String>,
    pub destination_extension: Option<String>,
    pub quality: Option<f32>,
    pub delete: bool,
    pub resize: Option<u8>,
    pub trim_xy: Option<(u32, u32)>,
    pub trim_wh: Option<(u32, u32)>,
    pub grayscale: bool,
    pub view: bool,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source file path
    source: String,

    /// Destination file path
    #[arg(short, long)]
    to: Option<String>,

    /// Destination file extension
    #[arg(short, long)]
    convert: Option<String>,

    /// Resize image
    #[arg(short, long)]
    resize: Option<u8>,

    /// Trim image
    #[arg(short, long)]
    trim: Option<String>,

    /// Grayscale image
    #[arg(short, long)]
    grayscale: bool,

    /// Image quality
    #[arg(short, long)]
    quality: Option<f32>,

    /// Delete source file
    #[arg(short, long)]
    delete: bool,

    /// View result
    #[arg(short, long)]
    view: bool,
}

pub fn parser() -> ArgStruct {
    let args = Args::parse();

    /*
    let re = Regex::new(r"^\d*x\d*$").unwrap();
    let resize = if let Some(resize_str) = args.resize {
        if re.is_match(&resize_str) {
            let mut resize = resize_str.split("x");
            let width = resize.next().unwrap().parse::<usize>().unwrap() as u32;
            let height = resize.next().unwrap().parse::<usize>().unwrap() as u32;
            Some((width, height))
        }
        else {
            println!("Invalid resize format. Please use 'WxH' (e.g.1920x1080).");
            std::process::exit(1);
        }
    }
    else {
        None
    };
    */
    
    let ret = ArgStruct {
        souce_path: args.source,
        destination_path: args.to,
        destination_extension: args.convert,
        quality: args.quality,
        delete: args.delete,
        resize: args.resize,
        trim_xy: None,
        trim_wh: None,
        grayscale: args.grayscale,
        view: args.view,
    };

    if args.trim.is_some() {
        let re = Regex::new(r"^\d*x\d*\+\d*\+\d*$").unwrap();
        let trim = args.trim.unwrap();
        if re.is_match(&trim) {
            let mut trim = trim.split("x");
            let x = trim.next().unwrap().parse::<u32>().unwrap();
            let y = trim.next().unwrap().parse::<u32>().unwrap();
            ret.trim_xy = Some((x, y));
        }
        else {
            println!("Invalid trim format. Please use 'WxH' (e.g.1920x1080).");
            std::process::exit(1);
        }
    }

    ret
}
