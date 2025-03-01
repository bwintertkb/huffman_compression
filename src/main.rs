use huffc::{
    build_huffman_array,
    cli::{validate_inputs, Args, Mode},
    deserialze_huffman, encode_huffman_array,
    fs::{read_file, write_file},
    huff_encode_bitvec, serialize_huffman, tally_frequency,
};

use std::{ffi::OsString, io::Read};

use clap::Parser;

fn main() {
    let args = Args::parse();
    let mode = match validate_inputs(&args) {
        Ok(mode) => mode,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let buffer: &'static [u8] = match mode {
        Mode::Stdin => {
            let mut buffer = Vec::new();
            let mut stdin = std::io::stdin();
            if let Err(e) = stdin.read_to_end(&mut buffer) {
                println!("Failed to read stdin: {}", e);
                return;
            }
            Box::leak(buffer.into_boxed_slice())
        }
        Mode::FileIO => {
            let path_buffer = args.input.as_ref().unwrap();
            let mmap = read_file(path_buffer);
            Box::leak(Box::new(mmap))
        }
    };

    if args.compress {
        let freq_buff = tally_frequency(buffer);
        let huffnode = build_huffman_array(freq_buff);
        let encoded_map = encode_huffman_array(&huffnode);
        let (bit_buffer, total_bits) = huff_encode_bitvec(buffer, &encoded_map);
        let serialized_buffer = serialize_huffman(&encoded_map, bit_buffer, total_bits);
        let base_file_path = match mode {
            Mode::Stdin => args.out_file.as_ref().unwrap(),
            Mode::FileIO => {
                if let Some(ref out_file) = args.out_file {
                    out_file
                } else {
                    args.input.as_ref().unwrap()
                }
            }
        };

        let mut write_file_path: OsString = base_file_path.into();
        write_file_path.push(".huff");

        write_file(write_file_path, serialized_buffer);
    } else if args.decompress {
        let base_file_path = match mode {
            Mode::Stdin => args.out_file.as_ref().unwrap(),
            Mode::FileIO => {
                if let Some(ref out_file) = args.out_file {
                    out_file
                } else {
                    args.input.as_ref().unwrap()
                }
            }
        };

        let mut base_file_clone = base_file_path.clone();

        if base_file_clone.extension().unwrap() == "huff" {
            base_file_clone.set_extension("");
        }

        let deserialized_bytes = deserialze_huffman(buffer);

        write_file(base_file_clone, deserialized_bytes);
    }
}
