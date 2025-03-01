use std::{
    fs::{File, OpenOptions},
    path::Path,
};

use memmap2::{Mmap, MmapMut};

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

pub fn read_file<P: AsRef<Path>>(path: P) -> Mmap {
    let file = File::open(path).unwrap();

    unsafe { Mmap::map(&file) }.unwrap()
}
