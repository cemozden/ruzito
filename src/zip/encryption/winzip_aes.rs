use std::io::{Error, Read, Write};
use std::cmp::Ordering;
use crypto::{aes::{KeySize, ctr}, hmac::Hmac, mac::Mac, pbkdf2::pbkdf2, sha1::Sha1};
use byteorder::{ByteOrder, LittleEndian};

use crate::zip::mem_map::AesEncryptionStrength;

const PBKDF2_ROUNDS: u32 = 1000;
const AES_EXTRA_FIELD_SIZE: usize = 11;
const AES_CTR_BUFFER_SIZE: usize = 16;

#[derive(Debug)]
pub enum WinZipAesError {
    IOError(Error),
    ExtraFieldSizeError(usize),
    UnknownEncryptionStrength(u8),
    InvalidPassword(String)
}

pub struct WinZipAesReader<R: Read> {
    reader: Box<R>,
    encryption_key: Vec<u8>,
    ctr: u32,
    ctr_bytes_remaining: usize,
    key_size: KeySize,
    iv: Vec<u8>,
    hmac: Hmac<Sha1> 
}

impl<R: Read> WinZipAesReader<R> {
    pub fn new(password: String, zip_item_extra_field: &[u8], auth_code: &mut [u8], reader: R) -> Result<Self, WinZipAesError> {

        let extra_field_length = zip_item_extra_field.len();

        if extra_field_length < AES_EXTRA_FIELD_SIZE {
            return Err(WinZipAesError::ExtraFieldSizeError(extra_field_length))
        }
        let encryption_strength_byte = zip_item_extra_field[8];
        let encryption_strength = AesEncryptionStrength::from_byte(encryption_strength_byte);

        let salt_size = match encryption_strength {
            AesEncryptionStrength::Aes128 => 8usize,
            AesEncryptionStrength::Aes192 => 12,
            AesEncryptionStrength::Aes256 => 16,
            AesEncryptionStrength::Unknown => return Err(WinZipAesError::UnknownEncryptionStrength(encryption_strength_byte))
        };
        let key_length = match encryption_strength {
            AesEncryptionStrength::Aes128 => 16,
            AesEncryptionStrength::Aes192 => 24,
            AesEncryptionStrength::Aes256 => 32,
            AesEncryptionStrength::Unknown => return Err(WinZipAesError::UnknownEncryptionStrength(encryption_strength_byte))
        };
        let key_size = match encryption_strength {
            AesEncryptionStrength::Aes128 => KeySize::KeySize128,
            AesEncryptionStrength::Aes192 => KeySize::KeySize192,
            AesEncryptionStrength::Aes256 => KeySize::KeySize256,
            AesEncryptionStrength::Unknown => return Err(WinZipAesError::UnknownEncryptionStrength(encryption_strength_byte))
        };
        let dk_len = key_length * 2 + 2;

        let mut key = vec![0u8; dk_len];
        let mut salt = vec![0u8; salt_size];
        let mut pass_verification_value = [0; 2];

        //TODO: Set reader with take option. That is, read until authentication_code

        let mut self_obj = WinZipAesReader {
            reader: Box::new(reader),
            encryption_key: Vec::new(),
            ctr: 1,
            key_size,
            ctr_bytes_remaining: 0,
            iv: vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            hmac: Hmac::new(Sha1::new(), &[])
        };

        self_obj.reader.read_exact(&mut salt)
            .map_err(|err| WinZipAesError::IOError(err))?;
        self_obj.reader.read_exact(&mut pass_verification_value)
            .map_err(|err| WinZipAesError::IOError(err))?;

        pbkdf2(&mut Hmac::new(Sha1::new(), password.as_bytes()), &salt, PBKDF2_ROUNDS, &mut key);

        let pass_verify_start_index = key_length * 2;
        auth_code.clone_from_slice(&key[key_length..pass_verify_start_index]);

        let hmac = Hmac::new(Sha1::new(), auth_code); 
        let pass_verification_value_from_key= &key[pass_verify_start_index..];

        if pass_verification_value_from_key != pass_verification_value {
            return Err(WinZipAesError::InvalidPassword(password));
        }
        let encryption_key = Vec::from(&key[..key_length]);
        self_obj.encryption_key = encryption_key;
        self_obj.hmac = hmac;

       Ok(self_obj)
    }

}

impl<R: Read> Read for WinZipAesReader<R> {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {

        let encrypted_buffer_len = self.reader.read(&mut buf)?;
        let mut bytes_decrypted = 0usize;
        let remaining_byte_count = self.ctr_bytes_remaining;

        if encrypted_buffer_len == 0 {
            let mut o = [0; 32];
            self.hmac.raw_result(&mut o);
            return Ok(encrypted_buffer_len)
        }

        if self.ctr_bytes_remaining > 0 {
            
            let mut aes_ctr = ctr(self.key_size, &self.encryption_key, &self.iv); 

            if encrypted_buffer_len < self.ctr_bytes_remaining {
                let mut decrypted_buffer = vec![0; self.ctr_bytes_remaining];
                let skipped_bytes_count = AES_CTR_BUFFER_SIZE - encrypted_buffer_len;
                let mut buffer_to_decrypt = [vec![0; skipped_bytes_count].as_slice(), buf].concat();

                aes_ctr.process(&mut buffer_to_decrypt, &mut decrypted_buffer);
                buf.write_all(&mut decrypted_buffer[skipped_bytes_count..])?;

                self.ctr += 1;
                self.ctr_bytes_remaining = 0;
                self.hmac.input(&buffer_to_decrypt);

                LittleEndian::write_u32(&mut self.iv, self.ctr);
                
                return Ok(encrypted_buffer_len)
            }
            else {
                let mut decrypted_buffer = vec![0; AES_CTR_BUFFER_SIZE];
                let skipped_bytes_count = AES_CTR_BUFFER_SIZE - self.ctr_bytes_remaining;
                let mut buffer_to_decrypt = [vec![0; skipped_bytes_count].as_slice(), &buf[..self.ctr_bytes_remaining]].concat();
                aes_ctr.process(&mut buffer_to_decrypt, &mut decrypted_buffer);
                
                let decrypted_bytes_remaining = &decrypted_buffer[skipped_bytes_count..];
                (0..self.ctr_bytes_remaining).into_iter()
                    .for_each(|index| buf[index] = decrypted_bytes_remaining[index]);

                self.ctr += 1;
                bytes_decrypted += self.ctr_bytes_remaining;
                self.ctr_bytes_remaining = 0;
                self.hmac.input(&buffer_to_decrypt);

                LittleEndian::write_u32(&mut self.iv, self.ctr);
            }

        }
        let bytes_to_decrypt_left = encrypted_buffer_len - bytes_decrypted;

        match bytes_to_decrypt_left.cmp(&AES_CTR_BUFFER_SIZE) {
            Ordering::Less => {
                let mut buffer_to_decrypt = &buf[bytes_decrypted..];
                let buffer_len = buffer_to_decrypt.len();
                let mut decrypted_buffer = vec![0; buffer_len];
                let mut aes_ctr = ctr(self.key_size, &self.encryption_key, &self.iv); 

                aes_ctr.process(&mut buffer_to_decrypt, &mut decrypted_buffer);

                (bytes_decrypted..buffer_len).into_iter()
                    .for_each(|index| buf[index] = decrypted_buffer[index]);

                bytes_decrypted += buffer_len;
            },
            Ordering::Equal => {
                let mut buffer_to_decrypt = &buf[bytes_decrypted..];
                let buffer_len = buffer_to_decrypt.len();
                let mut decrypted_buffer = [0; AES_CTR_BUFFER_SIZE];
                let mut aes_ctr = ctr(self.key_size, &self.encryption_key, &self.iv); 

                aes_ctr.process(&mut buffer_to_decrypt, &mut decrypted_buffer);

                (bytes_decrypted..buffer_len).into_iter()
                    .for_each(|index| buf[index] = decrypted_buffer[index]);

                self.ctr += 1;
                self.ctr_bytes_remaining = 0;

                LittleEndian::write_u32(&mut self.iv, self.ctr);

                bytes_decrypted += buffer_len;

            },
            Ordering::Greater => {
                let bytes_to_decrypt_later = bytes_to_decrypt_left % AES_CTR_BUFFER_SIZE;
                let ctr_cycle = (bytes_to_decrypt_left - bytes_to_decrypt_later) / AES_CTR_BUFFER_SIZE;

                (0..ctr_cycle).into_iter()
                    .for_each(|index| {
                        let buffer_slice_start_index = index * AES_CTR_BUFFER_SIZE + remaining_byte_count;
                        let buffer_slice_end_index = buffer_slice_start_index + AES_CTR_BUFFER_SIZE;
                        let mut buffer_to_decrypt = &buf[buffer_slice_start_index..buffer_slice_end_index];
                        let mut decrypted_buffer = [0; AES_CTR_BUFFER_SIZE];
                        let mut aes_ctr = ctr(self.key_size, &self.encryption_key, &self.iv); 

                        self.hmac.input(&buffer_to_decrypt);
                        aes_ctr.process(&mut buffer_to_decrypt, &mut decrypted_buffer);
                            
                        (0..AES_CTR_BUFFER_SIZE).into_iter()
                            .for_each(|index| buf[buffer_slice_start_index + index] = decrypted_buffer[index]);
                            
                        self.ctr += 1;
                        LittleEndian::write_u32(&mut self.iv, self.ctr);

                        bytes_decrypted += AES_CTR_BUFFER_SIZE;
                    });

                if bytes_to_decrypt_later > 0 {
                    let buffer_size = encrypted_buffer_len - bytes_to_decrypt_later;
                    let mut buffer_to_decrypt = &buf[buffer_size..encrypted_buffer_len];
                    let buffer_len = buffer_to_decrypt.len();
                    let mut decrypted_buffer = vec![0; bytes_to_decrypt_later];
                    let mut aes_ctr = ctr(self.key_size, &self.encryption_key, &self.iv); 

                    aes_ctr.process(&mut buffer_to_decrypt, &mut decrypted_buffer);


                    (0..bytes_to_decrypt_later).into_iter()
                            .for_each(|index| buf[buffer_size + index] = decrypted_buffer[index]);

                    self.ctr_bytes_remaining = AES_CTR_BUFFER_SIZE - bytes_to_decrypt_later;

                    bytes_decrypted += buffer_len;
                }

            }
        }

        Ok(bytes_decrypted)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;
    use byteorder::{ByteOrder, LittleEndian};

    #[test]
    fn test() {
        let mut buf = [0; 16];
        let mut buf2 = [0; 4];
        let zip_item_extra_field = [0x01, 0x99, 0x07, 0x00, 0x02, 0x00, 0x41, 0x45, 0x03, 0x08, 0x00];
        let reader = Cursor::new(
            vec![0x74, 0x68, 0x4C, 0x2D, 0x34, 0x98, 0xB2, 0x43, 0xC2, 0xD5, 0xFF, 0x26, 0x6F, 0x01, 0x60, 0x41, 0x8A, 0x34, 0x00, 0x32, 0x3A, 0x0D, 0xAB, 0xF5, 0xF6, 0x58, 0xA6, 0xA0, 0xCB, 0x08, 0x90, 0x62, 0xAA, 0xBB, 0x10, 0x6D, 0xF6, 0x62]
        );
        let mut auth_code = [0; 32];
        let mut winzip_aes_reader = WinZipAesReader::new(String::from("123456"),  &zip_item_extra_field, &mut auth_code, reader).unwrap();

        let count = winzip_aes_reader.read(&mut buf).unwrap();
        let count2 = winzip_aes_reader.read(&mut buf2).unwrap();

        println!("{:x?}", buf);
        println!("Count: {}", count);
        println!("ctr: {}", winzip_aes_reader.ctr);
        println!("ctr_bytes_remaining: {}\n", winzip_aes_reader.ctr_bytes_remaining);
        println!("{:x?}", buf2);
        println!("Count: {}", count2);
        println!("ctr: {}", winzip_aes_reader.ctr);
        println!("ctr_bytes_remaining: {}\n", winzip_aes_reader.ctr_bytes_remaining);
    }

    #[test]
    fn test1() {
        let mut x = vec![0u8; 16];
        x[0] = 1;
        LittleEndian::write_u32(&mut x, 1000);

        println!("{:x?}", x);
    }

}
