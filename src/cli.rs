//! Huffc - Huffman Compression CLI Tool
//!
//! This tool provides lossless compression and decompression using Huffman coding.
//! It supports reading from files and standard input and allows specifying output destinations.
//!
//! ## Usage
//!
//! - To compress a file:
//!   ```sh
//!   huffc --compress -i input.txt -o output.huff
//!   ```
//!
//! - To decompress a file:
//!   ```sh
//!   huffc --decompress -i output.huff -o original.txt
//!   ```
//!
//! - To compress data from stdin:
//!   ```sh
//!   cat input.txt | huffc --compress -o output.huff
//!   ```
//!
//! ## Error Handling
//!
//! The tool provides detailed error messages when invalid arguments are provided.
//!
//! ## Features
//! - Supports file-based and stdin input
//! - Provides informative error messages
//! - Enforces correct file extensions during decompression
//! - Uses `clap` for command-line argument parsing
//!
use std::{error::Error, fmt::Display, path::PathBuf};

use atty::Stream;
use clap::Parser;

/// Enum representing the mode of operation for the Huffman compression tool.
#[derive(Debug)]
pub enum Mode {
    /// Read input from stdin.
    Stdin,
    /// Read input from a file.
    FileIO,
}

/// Command-line argument parser for the Huffman compression tool.
#[derive(Debug, Parser)]
#[command(version, about="Huffman compression tool", long_about=None)]
pub struct Args {
    /// Flag to enable compression mode.
    #[arg(short, long)]
    pub compress: bool,
    /// Flag to enable decompression mode.
    #[arg(short, long)]
    pub decompress: bool,
    /// Optional input file path.
    #[arg(short, long, value_name = "INPUT", required = false)]
    pub input: Option<PathBuf>,
    /// Optional output file path.
    #[arg(short, long)]
    pub out_file: Option<PathBuf>,
}

/// Validates the command-line arguments and determines the operation mode.
///
/// # Arguments
///
/// * `args` - A reference to the parsed command-line arguments.
///
/// # Returns
///
/// * `Ok(Mode)` - If the arguments are valid, returns the corresponding mode.
/// * `Err(HuffErr)` - If invalid arguments are provided, returns an error.
pub fn validate_inputs(args: &Args) -> Result<Mode, HuffErr> {
    // Ensure that either --compress or --decompress is specified, but not both.
    if !(args.compress ^ args.decompress) {
        return Err(HuffErr::CompressionFlag);
    }

    // Check if input is coming from stdin.
    if !atty::is(Stream::Stdin) {
        // If reading from stdin, an output file must be specified.
        if args.out_file.is_none() {
            return Err(HuffErr::NoOutfileProvided);
        }
        return Ok(Mode::Stdin);
    }

    // If no input file is specified, return an error.
    if args.input.is_none() {
        return Err(HuffErr::NoFilePath);
    }

    if let Some(ref path) = args.input {
        // Check if the specified input file exists.
        if !path.exists() {
            return Err(HuffErr::FileDoesNotExist);
        }

        // If decompression mode is selected, ensure the file has the correct extension.
        if args.decompress && path.extension().unwrap() != "huff" {
            return Err(HuffErr::WrongFileExtension);
        }
    }

    Ok(Mode::FileIO)
}

/// Custom error type for handling argument validation errors.
#[derive(Debug)]
pub enum HuffErr {
    /// No arguments were provided.
    NoArgs,
    /// No file path was provided.
    NoFilePath,
    /// The specified file does not exist.
    FileDoesNotExist,
    /// The provided file has an incorrect extension.
    WrongFileExtension,
    /// No output file was provided for stdin input.
    NoOutfileProvided,
    /// No valid arguments were provided.
    NoValidArgs,
    /// Both compression and decompression flags were set.
    CompressionFlag,
}

/// Implement the `Display` trait to provide user-friendly error messages.
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

/// Implement the `Error` trait for `HuffErr` to allow integration with Rust's error handling.
impl Error for HuffErr {}
