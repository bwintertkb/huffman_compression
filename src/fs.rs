//! Huffc File IO - File Reading and Writing Utility
//!
//! This module provides efficient file reading and writing using memory-mapped files.
//! It ensures fast data access and modification without excessive system calls.
//!
//! ## Features
//! - Uses `memmap2` for efficient file I/O operations.
//! - Supports reading and writing large files efficiently.
//! - Ensures safe memory mapping with flush operations.
//!
//! ## Usage
//!
//! - Writing to a file:
//!   ```rust
//!   use huffc::fs::write_file;
//!   write_file("output.huff", vec![1, 2, 3, 4, 5]);
//!   ```
//!
//! - Reading from a file:
//!   ```rust
//!   use huffc::fs::read_file;
//!   let data = read_file("./tests/resources/input.txt");
//!   println!("File contents: {:?}", &data[..]);
//!   ```
//!
use std::{
    fs::{File, OpenOptions},
    path::Path,
};

use memmap2::{Mmap, MmapMut};

/// Writes a buffer to a file using memory-mapped I/O.
///
/// # Arguments
///
/// * `path` - Path to the file to be written.
/// * `buffer` - Data to be written into the file.
///
/// This function creates or truncates the file, maps it into memory,
/// and writes the data efficiently before flushing changes to disk.
///
pub fn write_file<P: AsRef<Path>>(path: P, buffer: Vec<u8>) {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();

    file.set_len(buffer.len() as u64).unwrap();

    let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };
    mmap[..].copy_from_slice(&buffer);
    mmap.flush().unwrap();
}

/// Reads a file into a memory-mapped buffer.
///
/// # Arguments
///
/// * `path` - Path to the file to be read.
///
/// # Returns
///
/// * `Mmap` - Memory-mapped buffer containing the file contents.
///
/// This function opens a file and maps its contents into memory,
/// allowing efficient access without excessive system calls.
///
pub fn read_file<P: AsRef<Path>>(path: P) -> Mmap {
    let file = File::open(path).unwrap();

    unsafe { Mmap::map(&file) }.unwrap()
}
