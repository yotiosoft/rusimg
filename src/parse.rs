use clap::Parser;

pub struct ArgStruct {
    pub souce_path: String,
    pub destination_path: Option<String>,
    pub destination_extension: Option<String>,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source file path
    source: String,

    /// Destination file path
    #[arg(short, long)]
    destination: Option<String>,

    /// Destination file extension
    #[arg(short, long)]
    extension: Option<String>,
}

pub fn parser() -> ArgStruct {
    let args = Args::parse();
    let mut arg_struct = ArgStruct {
        souce_path: args.source,
        destination_path: None,
        destination_extension: None,
    };

    if let Some(destination) = args.destination {
        arg_struct.destination_path = Some(destination);
    }

    if let Some(extension) = args.extension {
        arg_struct.destination_extension = Some(extension);
    }

    arg_struct
}
