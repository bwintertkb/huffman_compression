use huffman_compression::{
    build_huffman_array,
    cli::{validate_inputs, Args, Mode},
    deserialze_huffman, encode_huffman_array,
    fs::{read_file, write_file},
    huff_encode_bitvec, serialize_huffman, tally_frequency,
};
use memmap2::Mmap;

use std::{ffi::OsString, fs::File, io::Read};

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

    let buffer = match mode {
        Mode::Stdin => {
            let mut buffer = Vec::new();
            let mut stdin = std::io::stdin();
            if let Err(e) = stdin.read_to_end(&mut buffer) {
                println!("Failed to read stdin: {}", e);
                return;
            }
            buffer
        }
        Mode::FileIO => {
            let path_buffer = args.input.as_ref().unwrap();
            let file = File::open(path_buffer).unwrap();
            let mmap = unsafe { Mmap::map(&file).unwrap() };
            mmap.to_vec()
        }
    };

    if args.compress {
        let bytes = buffer.as_slice();
        let freq_buff = tally_frequency(bytes);
        let huffnode = build_huffman_array(freq_buff);
        let encoded_map = encode_huffman_array(&huffnode);
        let (bit_buffer, total_bits) = huff_encode_bitvec(bytes, &encoded_map);
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

        let mmap = read_file(base_file_path);

        let deserialized_bytes = deserialze_huffman(&mmap[..]);

        write_file(base_file_clone, deserialized_bytes);
    }
}
