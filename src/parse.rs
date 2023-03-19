use clap::{Parser, Subcommand};

#[derive(PartialEq)]
pub enum ExecutionMode {
    Compress,
    Convert,
    None,
}

pub struct ArgStruct {
    pub execution_mode: ExecutionMode,
    pub souce_path: String,
    pub destination_path: Option<String>,
    pub destination_extension: Option<String>,
    pub quality: Option<f32>,
    pub delete: bool,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Subcommands
    #[clap(subcommand)]
    subcmds: Option<SubCommands>,
}

#[derive(Debug, Subcommand)]
enum SubCommands {
    /// Compress image
    Compress {
        /// Source file path
        source: String,

        /// Destination file path
        #[arg(short, long)]
        to: Option<String>,
    },

    /// Image conversion
    Convert {
        /// Source file path
        source: String,

        /// Destination file path
        #[arg(short, long)]
        to: Option<String>,

        /// Destination file extension
        #[arg(short, long)]
        extension: String,

        /// Image quality
        #[arg(short, long)]
        quality: Option<f32>,

        /// Delete source file
        #[arg(short, long)]
        delete: bool,
    },
}

pub fn parser() -> ArgStruct {
    let args = Args::parse();
    let mut arg_struct = ArgStruct {
        execution_mode: ExecutionMode::None,
        souce_path: String::new(),
        destination_path: None,
        destination_extension: None,
        quality: None,
        delete: false,
    };

    // Subcommands
    if let Some(subcmds) = args.subcmds {
        match subcmds {
            SubCommands::Compress { source, to } => {
                arg_struct.execution_mode = ExecutionMode::Compress;
                arg_struct.souce_path = source;
                if let Some(destination) = to {
                    arg_struct.destination_path = Some(destination);
                }
            }
            SubCommands::Convert { source, to, extension, quality, delete, } => {
                arg_struct.execution_mode = ExecutionMode::Convert;
                arg_struct.souce_path = source;
                if let Some(destination) = to {
                    arg_struct.destination_path = Some(destination);
                }
                arg_struct.destination_extension = Some(extension);
                if let Some(quality) = quality {
                    arg_struct.quality = Some(quality);
                }
                arg_struct.delete = delete;
            }
        }
    }

    arg_struct
}
