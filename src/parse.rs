use clap::Parser;

pub struct ArgStruct {
    pub souce_path: String,
    pub destination_path: Option<String>,
    pub destination_extension: Option<String>,
    pub quality: Option<f32>,
    pub delete: bool,
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

    /// Image quality
    #[arg(short, long)]
    quality: Option<f32>,

    /// Delete source file
    #[arg(short, long)]
    delete: bool,
}

pub fn parser() -> ArgStruct {
    let args = Args::parse();
    
    ArgStruct {
        souce_path: args.source,
        destination_path: args.to,
        destination_extension: args.convert,
        quality: args.quality,
        delete: args.delete,
    }
}
