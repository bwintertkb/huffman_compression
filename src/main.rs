use huffman_compression::{
    build_huffman_tree,
    cli::{validate_inputs, Args, Mode},
    deserialze_huffman, generate_encode_map, huff_encode_bitvec, serialize_huffman,
    tally_frequency,
};
use memmap2::{Mmap, MmapMut};

use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::Read,
};

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
            let path_buffer = args.read_file_path.as_ref().unwrap();
            let file = File::open(path_buffer).unwrap();
            let mmap = unsafe { Mmap::map(&file).unwrap() };
            mmap.to_vec()
        }
    };

    if args.compress {
        let bytes = buffer.as_slice();
        let freq_buff = tally_frequency(bytes);
        let huffnode = build_huffman_tree(freq_buff);
        let encoded_map = generate_encode_map(huffnode.unwrap());
        let (bit_buffer, total_bits) = huff_encode_bitvec(bytes, &encoded_map);
        let serialized_buffer = serialize_huffman(&encoded_map, bit_buffer, total_bits);

        let base_file_path = match mode {
            Mode::Stdin => args.out_file.as_ref().unwrap(),
            Mode::FileIO => {
                if let Some(ref out_file) = args.out_file {
                    out_file
                } else {
                    args.read_file_path.as_ref().unwrap()
                }
            }
        };

        let mut write_file_path: OsString = base_file_path.into();
        write_file_path.push(".huff");

        let new_file_path = base_file_path.with_file_name(write_file_path);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(new_file_path)
            .unwrap();

        let file_size = serialized_buffer.len();
        file.set_len(file_size as u64).unwrap();

        let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };
        mmap[..].copy_from_slice(&serialized_buffer);

        mmap.flush().unwrap();
    } else if args.decompress {
        let base_file_path = match mode {
            Mode::Stdin => args.out_file.as_ref().unwrap(),
            Mode::FileIO => {
                if let Some(ref out_file) = args.out_file {
                    out_file
                } else {
                    args.read_file_path.as_ref().unwrap()
                }
            }
        };

        let mut base_file_clone = base_file_path.clone();

        if base_file_clone.extension().unwrap() == "huff" {
            base_file_clone.set_extension("");
        }

        // TODO Make file reads mmaps and make it work when decompressing from stdin
        let file_contents = std::fs::read(base_file_path).unwrap();

        let deserialized_bytes = deserialze_huffman(&file_contents);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(base_file_clone)
            .unwrap();

        let file_size = deserialized_bytes.len();
        file.set_len(file_size as u64).unwrap();

        let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };
        mmap[..].copy_from_slice(&deserialized_bytes);
        mmap.flush().unwrap();
    }
}
