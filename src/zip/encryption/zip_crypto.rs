use crc::crc32::make_table;
use std::num::Wrapping;
use std::io::{Read, Error};
use byteorder::{ByteOrder, BigEndian};

const PKZIP_INITIAL_KEY_1: u32 = 0x12345678;
const PKZIP_INITIAL_KEY_2: u32 = 0x23456789;
const PKZIP_INITIAL_KEY_3: u32 = 0x34567890;
const ZIP_CRYPTO_POLYNOMIAL: u32 = 0xEDB88320;

pub struct ZipCryptoReader<R: Read> {
    key1: Wrapping<u32>,
    key2: Wrapping<u32>,
    key3: Wrapping<u32>,
    zip_crypto_polynomial_table: [u32; 256],
    reader: Box<R>
}

#[derive(Debug)]
pub enum ZipCryptoError {
    InvalidPassword(String),
    IOError(Error)
}

impl<R: Read> ZipCryptoReader<R> {
    pub fn new(password: String, file_crc: u32, reader: R) -> Result<Self, ZipCryptoError> {
        let polynomial_table = make_table(ZIP_CRYPTO_POLYNOMIAL);

        let mut self_obj = Self {
           key1: Wrapping(PKZIP_INITIAL_KEY_1),
           key2: Wrapping(PKZIP_INITIAL_KEY_2),
           key3: Wrapping(PKZIP_INITIAL_KEY_3),
           zip_crypto_polynomial_table: polynomial_table,
           reader: Box::new(reader)
        };

        let mut encryption_header = vec![0; 12];

        if let Err(err) = self_obj.reader.read_exact(&mut encryption_header) {
            return Err(ZipCryptoError::IOError(err));
        }

        encryption_header = self_obj.decrypt_encryption_header(&password, &encryption_header);
        let mut crc_bytes = [0; 4];
        BigEndian::write_u32(&mut crc_bytes, file_crc);
        let crc_high_order_byte = &crc_bytes[0];

        if &encryption_header[11] != crc_high_order_byte {
            return Err(ZipCryptoError::InvalidPassword(password))
        }

        Ok(self_obj)
    }

    fn decrypt_encryption_header(&mut self, password: &String, encryption_header: &[u8]) -> Vec<u8> {

        password.as_bytes().into_iter()
            .for_each(|ch| { self.update_keys(*ch) } );

        encryption_header.iter()
            .map(|byte| {
                let ch_byte = *byte ^ self.stream_byte();
                self.update_keys(ch_byte);

                ch_byte
            }).collect()
    }

    fn decrypt_byte(&mut self, char_byte: u8) -> u8 {

        let temp = self.stream_byte() ^ char_byte;
        self.update_keys(temp);

        temp
    }

    fn encrypt_byte(&mut self, char_byte: u8) -> u8 {
        let cipher_byte = self.stream_byte() ^ char_byte;
        self.update_keys(char_byte);

        cipher_byte
    }

    fn update_keys(&mut self, ch: u8) {
        
        self.key1 = self.crc32(self.key1, ch);
        self.key2 = (self.key2 + (self.key1 & Wrapping(0xff))) * Wrapping(0x08088405) + Wrapping(1);
        self.key3 = self.crc32(self.key3, (self.key2 >> 24).0 as u8);
    }

    fn crc32(&self, crc: Wrapping<u32>, input: u8) -> Wrapping<u32> {
       (crc >> 8) ^ Wrapping(self.zip_crypto_polynomial_table[((crc & Wrapping(0xff)).0 as u8 ^ input) as usize])
    }

    fn stream_byte(&self) -> u8 {
        let temp: Wrapping<u16> = Wrapping(self.key3.0 as u16) | Wrapping(3);
        ((temp * (temp ^ Wrapping(1))) >> 8).0 as u8
    }
}

impl<R: Read> Read for ZipCryptoReader<R> {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        let read_buf_size = match self.reader.read(&mut buf) {
            Ok(size) => size,
            Err(err) => return Err(err)
        };

        buf.iter_mut().take(read_buf_size).for_each(|byte| { *byte = self.decrypt_byte(*byte) });

        Ok(read_buf_size)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;

    #[test]
    fn should_yield_error_if_password_is_wrong() {
        let cursor = Cursor::new([0xD0, 0x66, 0x78, 0x57, 0xA6, 0xC0, 0x45, 0x75, 0x7B, 0x0F, 0x77, 0x8F, 0x36, 0x53, 0x9b, 0x6f, 0xAC]);
        let zip_crypto_encryption_reader = ZipCryptoReader::new(String::from("1234567"), 
        0x2952CCF, 
        cursor);

        assert!(zip_crypto_encryption_reader.is_err());
    }

    #[test]
    fn should_yield_zip_crypto_reader_if_password_is_correct() {
        let cursor = Cursor::new([0xD0, 0x66, 0x78, 0x57, 0xA6, 0xC0, 0x45, 0x75, 0x7B, 0x0F, 0x77, 0x8F, 0x36, 0x53, 0x9b, 0x6f, 0xAC]);
        let mut zip_crypto_encryption_reader = ZipCryptoReader::new(String::from("123456"), 
        0x2952CCF, 
        cursor).unwrap();

        let mut buf = [0; 4];

        let _ = zip_crypto_encryption_reader.read(&mut buf);
        
        assert_eq!(buf, [0xB5, 0x5B, 0x4B, 0x72]);
    }

}