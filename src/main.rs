
use std::path::Path;
use std::io::Read;
use std::io::ErrorKind;
use std::io::Error;
extern crate byteorder;

use std::fs::File;
use std::fs::metadata;
use byteorder::{LittleEndian, ByteOrder};

mod zip;


fn main() {
    let zip_file_path = r"C:\eula.zip";
    let buffer = read_bin(zip_file_path);
    let mut slice = vec!([0x14, 0x40]);
    println!("{:?}", LittleEndian::read_u16(&mut slice[0]));
    //println!("{:?}", buffer.unwrap());
}

#[test]
fn test_read_zip_file_fails_for_unknown_paths() {
    let file_path = r"unknown_path";
    let read_bin_result = read_bin(file_path); 

    assert!(read_bin_result.is_err());
    assert_eq!(read_bin_result.err().unwrap().kind(), ErrorKind::NotFound);
}

fn read_bin<P: AsRef<Path>>(file_path: P) -> Result<Box<[u8]>, Error> {
    let mut zip_file = File::open(&file_path)?;
    let file_metadata = metadata(&file_path)?;
    
    let mut buffer = Vec::with_capacity(file_metadata.len() as usize);
    zip_file.read_to_end(&mut buffer)?;
    
    Ok(buffer.into_boxed_slice())
}