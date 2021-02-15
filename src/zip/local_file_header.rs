use std::io::{Error, ErrorKind, SeekFrom};
use std::io::prelude::*;
use byteorder::{LittleEndian, ByteOrder};
use super::mem_map::{ZipVersion, CompressionMethod, FILE_HEADER_SIGNATURE, EncryptionMethod};
use super::date_time::ZipDateTime;

#[derive(Debug)]
pub struct LocalFileHeader {
    signature: u32,
    version_needed_to_extract: ZipVersion,
    general_purpose_flag: u16,
    encryption_method: EncryptionMethod,
    compression_method: CompressionMethod,
    last_modified_date_time: ZipDateTime,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    file_name_length: u16,
    file_name: String,
    extra_field_length: u16,
    extra_field: Vec<u8>,
    content_start_offset: u64
}

impl LocalFileHeader {
    pub fn from_reader<R>(reader: &mut R) -> Result<Self, Error>
    where R: Read + Seek {
        let start_offset = reader.seek(SeekFrom::Current(0)).unwrap();

        let mut cdf_bytes = vec![0; 30];
        reader.read_exact(&mut cdf_bytes)?;

        let reader_signature = LittleEndian::read_u32(&cdf_bytes[0..4]);
        if reader_signature != FILE_HEADER_SIGNATURE {
            return Err(Error::new(ErrorKind::InvalidData, format!("Invalid Local Header. {:#} ", FILE_HEADER_SIGNATURE)));
        }

        let file_name_length = LittleEndian::read_u16(&cdf_bytes[26..28]);
        let mut file_name_bytes: Vec<u8> = vec![0; file_name_length as usize];

        let extra_field_length = LittleEndian::read_u16(&cdf_bytes[28..30]);
        let mut extra_field: Vec<u8> = vec![0; extra_field_length as usize];

        let general_purpose_flag = LittleEndian::read_u16(&cdf_bytes[6..8]);
        let content_start_offset = start_offset + 30 + (file_name_length + extra_field_length) as u64;
        let crc32 = LittleEndian::read_u32(&cdf_bytes[14..18]);
        let compressed_size = LittleEndian::read_u32(&cdf_bytes[18..22]);
        let uncompressed_size = LittleEndian::read_u32(&cdf_bytes[22..26]);
        let compression_method = CompressionMethod::from_addr(LittleEndian::read_u16(&cdf_bytes[8..10]));
        let encryption_method = if general_purpose_flag & 0b100001 == 0b100001 {
                EncryptionMethod::StrongEncryption
            }
            else if general_purpose_flag & 0x1 == 1 && compression_method != CompressionMethod::Aex {
                EncryptionMethod::ZipCrypto
            }
            else if compression_method == CompressionMethod::Aex { EncryptionMethod::WinZipAesEncryption }
            else { EncryptionMethod::NoEncryption };

            if general_purpose_flag >> 2 & 1 == 1 {
                return Err(Error::new(ErrorKind::InvalidData, "Data descriptor is currently not supported."));
            }

        reader.read_exact(&mut file_name_bytes)?;

        if extra_field_length > 0 {
            reader.read_exact(&mut extra_field)?;
        }

        Ok(LocalFileHeader {
            signature: FILE_HEADER_SIGNATURE,
            version_needed_to_extract:  ZipVersion::from_byte(cdf_bytes[4]),
            general_purpose_flag,
            compression_method,
            encryption_method,
            last_modified_date_time: ZipDateTime::from_addr(LittleEndian::read_u16(&cdf_bytes[12..14]), LittleEndian::read_u16(&cdf_bytes[10..12])),
            crc32,
            compressed_size,
            uncompressed_size,
            file_name_length,
            file_name: String::from_utf8(file_name_bytes).unwrap(),
            extra_field_length,
            extra_field,
            content_start_offset
        })
    }

    pub fn compression_method(&self) -> &CompressionMethod {
        &self.compression_method
    }

    pub fn content_start_offset(&self) -> u64 {
        self.content_start_offset
    }

    pub fn encryption_method(&self) -> &EncryptionMethod {
        &self.encryption_method
    }

    pub fn crc32(&self) -> u32 {
        return self.crc32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    #[should_panic]
    fn local_file_header_signature_valid() {
        let bytes = vec![0x50, 0x4B, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x34, 0xBE, 0x7D, 0x51, 0xCF, 0x2C, 0x95, 0x02, 0x10, 0x11, 0x00, 0x00, 0x46, 0x45, 0x00, 0x00,
        0x0D, 0x00, 0x00, 0x00, 0x65, 0x75, 0x6C, 0x61, 0x2E, 0x31, 0x30, 0x32, 0x38, 0x2E, 0x74, 0x78, 0x74];

        let mut cursor = Cursor::new(bytes);
        LocalFileHeader::from_reader(&mut cursor).unwrap();
    }

    #[test]
    fn parses_local_header_successfully() {
        let bytes = vec![0x50, 0x4B, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x34, 0xBE, 0x7D, 0x51, 0xCF, 0x2C, 0x95, 0x02, 0x10, 0x11, 0x00, 0x00, 0x46, 0x45, 0x00, 0x00,
        0x0D, 0x00, 0x00, 0x00, 0x65, 0x75, 0x6C, 0x61, 0x2E, 0x31, 0x30, 0x32, 0x38, 0x2E, 0x74, 0x78, 0x74];

        let mut cursor = Cursor::new(bytes);
        let local_file_header = LocalFileHeader::from_reader(&mut cursor).unwrap();

        assert_eq!(local_file_header.version_needed_to_extract, ZipVersion::from_byte(20));
        assert_eq!(local_file_header.general_purpose_flag, 0);
        assert_eq!(local_file_header.encryption_method, EncryptionMethod::NoEncryption);
        assert_eq!(local_file_header.compression_method, CompressionMethod::Deflate);
        assert_eq!(local_file_header.last_modified_date_time, ZipDateTime::new(29, 11, 2020, 23, 49, 40));
        assert_eq!(local_file_header.crc32, 43330767);
        assert_eq!(local_file_header.compressed_size, 4368);
        assert_eq!(local_file_header.uncompressed_size, 17734);
        assert_eq!(local_file_header.file_name_length, 13);
        assert_eq!(local_file_header.file_name, String::from("eula.1028.txt"));
    }
    #[test]
    fn parses_local_header_with_extra_field_successfully() {
        let bytes = vec![0x50, 0x4B, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x34, 0xBE, 0x7D, 0x51, 0xCF, 0x2C, 0x95, 0x02, 0x10, 0x11, 0x00, 0x00, 0x46, 0x45, 0x00, 0x00,
        0x0D, 0x00, 0x0B, 0x00, 0x65, 0x75, 0x6C, 0x61, 0x2E, 0x31, 0x30, 0x32, 0x38, 0x2E, 0x74, 0x78, 0x74, 0x01, 0x99, 0x07, 0x00, 0x02, 0x00, 0x41, 0x45, 0x03, 0x08, 0x00];

        let mut cursor = Cursor::new(bytes);
        let local_file_header = LocalFileHeader::from_reader(&mut cursor).unwrap();

        assert_eq!(local_file_header.version_needed_to_extract, ZipVersion::from_byte(20));
        assert_eq!(local_file_header.general_purpose_flag, 0);
        assert_eq!(local_file_header.encryption_method, EncryptionMethod::NoEncryption);
        assert_eq!(local_file_header.compression_method, CompressionMethod::Deflate);
        assert_eq!(local_file_header.last_modified_date_time, ZipDateTime::new(29, 11, 2020, 23, 49, 40));
        assert_eq!(local_file_header.crc32, 43330767);
        assert_eq!(local_file_header.compressed_size, 4368);
        assert_eq!(local_file_header.uncompressed_size, 17734);
        assert_eq!(local_file_header.file_name_length, 13);
        assert_eq!(local_file_header.file_name, String::from("eula.1028.txt"));
        assert_eq!(local_file_header.extra_field_length, 11);
        assert_eq!(local_file_header.extra_field, [0x01, 0x99, 0x07, 0x00, 0x02, 0x00, 0x41, 0x45, 0x03, 0x08, 0x00]);
    }
    #[test]
    fn returns_error_if_data_descriptor_specified() {
        let bytes = vec![0x50, 0x4B, 0x08, 0x04, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x34, 0xBE, 0x7D, 0x51, 0xCF, 0x2C, 0x95, 0x02, 0x10, 0x11, 0x00, 0x00, 0x46, 0x45, 0x00, 0x00,
        0x0D, 0x00, 0x00, 0x00, 0x65, 0x75, 0x6C, 0x61, 0x2E, 0x31, 0x30, 0x32, 0x38, 0x2E, 0x74, 0x78, 0x74];

        let mut cursor = Cursor::new(bytes);
        let local_file_header = LocalFileHeader::from_reader(&mut cursor);

        assert!(local_file_header.is_err());
        assert!(local_file_header.err().unwrap().kind() == ErrorKind::InvalidData)
    }

}