use std::{fs::File, io::{Read, Error}, path::PathBuf};
use crc::{crc32, Hasher32};

pub fn calculate_checksum(path: &PathBuf) -> Result<u32, Error> {
    let mut file = File::open(path)?;
    let mut buf = vec![0; 1_048_576];

    let mut digest = crc32::Digest::new(crc32::IEEE);

    while match file.read(&mut buf) {
        Ok(bytes_read) => {
            digest.write(&buf[0..bytes_read]);
            bytes_read > 0
        },
        Err(err) => return Err(err)
    } {}

    Ok(digest.sum32())
}