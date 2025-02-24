use std::{env, error::Error, fmt::Display, path::PathBuf};

use atty::Stream;
use clap::Parser;

const FLAG_COMPRESS: [&str; 2] = ["-c", "--compress"];
const FLAG_DECOMPRESS: [&str; 2] = ["-d", "--decompress"];
const FLAG_FILE_PATH: [&str; 2] = ["-f", "--file"];
const FLAG_OUTFILE: [&str; 2] = ["-o", "--out-file"];

const COMPRESS_VALUE: u8 = 0b00000001;
const DECOMPRESS_VALUE: u8 = 0b00000010;
const FILE_PATH_VALUE: u8 = 0b00000100;
const OUTFILE_VALUE: u8 = 0b00001000;
const STDIN_VALUE: u8 = 0b00010000;

#[derive(Debug)]
pub enum Mode {
    Stdin,
    FileIO,
}

#[derive(Debug, Parser)]
// #[command(version, Huffman compression tool, long_about=None)]
pub struct Args {
    #[arg(short, long)]
    pub compress: bool,
    #[arg(short, long)]
    pub decompress: bool,
    #[arg(short, long)]
    pub read_file_path: Option<PathBuf>,
    #[arg(short, long)]
    pub out_file: Option<PathBuf>,
}

pub fn validate_inputs(args: &Args) -> Result<Mode, HuffErr> {
    if !args.compress && !args.decompress {
        return Err(HuffErr::MissingCompressDecompress);
    }

    if args.compress && args.decompress {
        return Err(HuffErr::BothCompressDecompress);
    }

    if !atty::is(Stream::Stdin) {
        // Stdin detected - need to have outfile argument
        if args.out_file.is_none() {
            return Err(HuffErr::NoOutfileProvided);
        }

        return Ok(Mode::Stdin);
    }

    if args.read_file_path.is_none() {
        return Err(HuffErr::NoFilePath);
    }

    if let Some(ref path) = args.read_file_path {
        if !path.exists() {
            return Err(HuffErr::FileDoesNotExist);
        }

        if args.decompress && path.extension().unwrap() != "huff" {
            return Err(HuffErr::WrongFileExtension);
        }
    }

    Ok(Mode::FileIO)
}

// fn validate_inputs() -> Result<u8, HuffErr> {
//     let mut args_byte = 0;
//     if args.any(|arg| FLAG_OUTFILE.contains(&arg.as_str())) {
//         args_byte |= OUTFILE_VALUE;
//     }
//
//     if !atty::is(Stream::Stdin) {
//         return Ok(Mode::Stdin);
//     }
//
//     let mut args = env::args();
//     if args.len() < 2 {
//         // No args have been found, and no stdin, do not know what to do...
//         return Err(HuffErr::NoArgs);
//     }
//
//     if !args.any(|arg| FLAG_FILE_PATH.contains(&arg.as_str())) {
//         return Err(HuffErr::NoFilePath);
//     }
// }
//
#[derive(Debug)]
pub enum HuffErr {
    NoArgs,
    NoFilePath,
    FileDoesNotExist,
    WrongFileExtension,
    NoOutfileProvided,
    NoValidArgs,
    MissingCompressDecompress,
    BothCompressDecompress,
}

impl Display for HuffErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HuffErr::NoArgs => write!(f, "No arguments provided. Use --help for more information."),
            HuffErr::FileDoesNotExist => write!(f, "File does not exist in path provided. Use --help for more information."),
            HuffErr::WrongFileExtension => write!(f, "File does not have the file extension '.huff'. Use --help for more information."),
            HuffErr::BothCompressDecompress => write!(f, "Both compress and decompress were set, this is not allowed. Use --help for more information."),
            HuffErr::MissingCompressDecompress => write!(f, "Missing either compress or decompress. Use --help for more information."),
            HuffErr::NoFilePath => {
                write!(f, "No file path provided. Use --help for more information.")
            }
            HuffErr::NoValidArgs => write!(
                f,
                "No valid arguments provided. Use --help for more information."
            ),
            HuffErr::NoOutfileProvided => write!(f, "No outfile path provided. Use --help for more information.")
        }
    }
}

impl Error for HuffErr {}
//
// struct HuffBufferHandler {}
