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
        destination: Option<String>,
    },

    /// Image conversion
    Convert {
        /// Source file path
        source: String,

        /// Destination file path
        #[arg(short, long)]
        destination: Option<String>,

        /// Destination file extension
        #[arg(short, long)]
        extension: String,
    },
}

pub fn parser() -> ArgStruct {
    let args = Args::parse();
    let mut arg_struct = ArgStruct {
        execution_mode: ExecutionMode::None,
        souce_path: String::new(),
        destination_path: None,
        destination_extension: None,
    };

    // Subcommands
    if let Some(subcmds) = args.subcmds {
        match subcmds {
            SubCommands::Compress { source, destination } => {
                arg_struct.execution_mode = ExecutionMode::Compress;
                arg_struct.souce_path = source;
                if let Some(destination) = destination {
                    arg_struct.destination_path = Some(destination);
                }
            }
            SubCommands::Convert { source, destination, extension, } => {
                arg_struct.execution_mode = ExecutionMode::Convert;
                arg_struct.souce_path = source;
                if let Some(destination) = destination {
                    arg_struct.destination_path = Some(destination);
                }
                arg_struct.destination_extension = Some(extension);
            }
        }
    }

    arg_struct
}
