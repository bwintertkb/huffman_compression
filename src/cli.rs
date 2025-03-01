use std::{error::Error, fmt::Display, path::PathBuf};

use atty::Stream;
use clap::Parser;

#[derive(Debug)]
pub enum Mode {
    Stdin,
    FileIO,
}

#[derive(Debug, Parser)]
#[command(version, about="Huffman compression tool", long_about=None)]
pub struct Args {
    #[arg(short, long)]
    pub compress: bool,
    #[arg(short, long)]
    pub decompress: bool,
    #[arg(short, long, value_name = "INPUT", required = false)]
    pub input: Option<PathBuf>,
    #[arg(short, long)]
    pub out_file: Option<PathBuf>,
}

pub fn validate_inputs(args: &Args) -> Result<Mode, HuffErr> {
    if !(args.compress ^ args.decompress) {
        return Err(HuffErr::CompressionFlag);
    }

    if !atty::is(Stream::Stdin) {
        // Stdin detected - need to have outfile argument
        if args.out_file.is_none() {
            return Err(HuffErr::NoOutfileProvided);
        }

        return Ok(Mode::Stdin);
    }

    if args.input.is_none() {
        return Err(HuffErr::NoFilePath);
    }

    if let Some(ref path) = args.input {
        if !path.exists() {
            return Err(HuffErr::FileDoesNotExist);
        }

        if args.decompress {
            if path.extension().unwrap() != "huff" {
                return Err(HuffErr::WrongFileExtension);
            }
            if !path.exists() {
                return Err(HuffErr::FileDoesNotExist);
            }
        }
    }

    Ok(Mode::FileIO)
}

#[derive(Debug)]
pub enum HuffErr {
    NoArgs,
    NoFilePath,
    FileDoesNotExist,
    WrongFileExtension,
    NoOutfileProvided,
    NoValidArgs,
    CompressionFlag,
}

impl Display for HuffErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HuffErr::NoArgs => write!(f, "No arguments provided. Use --help for more information."),
            HuffErr::FileDoesNotExist => write!(
                f,
                "File does not exist in path provided. Use --help for more information."
            ),
            HuffErr::WrongFileExtension => write!(
                f,
                "File does not have the file extension '.huff'. Use --help for more information."
            ),
            HuffErr::CompressionFlag => write!(
                f,
                "Error: You must specify either --compress or --decompress, but not both.",
            ),
            HuffErr::NoFilePath => {
                write!(f, "No file path provided. Use --help for more information.")
            }
            HuffErr::NoValidArgs => write!(
                f,
                "No valid arguments provided. Use --help for more information."
            ),
            HuffErr::NoOutfileProvided => write!(
                f,
                "No outfile path provided. Use --help for more information."
            ),
        }
    }
}

impl Error for HuffErr {}
