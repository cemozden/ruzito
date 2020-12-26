use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::Error;
use super::mem_map::{HostOS, CENTRAL_DIR_SIGNATURE, ZipVersion, CompressionMethod};
use byteorder::{LittleEndian, ByteOrder};
use super::date_time::*;
use super::zip_item::ZipItem;

#[derive(Debug)]
pub struct CentralDirectoryFileHeader {
    signature: u32,
    host_os: HostOS,
    zip_specification: ZipVersion,
    version_needed_to_extract: ZipVersion,
    general_purpose_flag: u16,
    file_encrypted: bool,
    compression_method: CompressionMethod,
    last_modified_date_time: ZipDateTime,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    file_name_length: u16,
    extra_field_length: u16,
    file_comment_length: u16,
    disk_number_start: u16,
    internal_file_attr: u16,
    external_file_attr: u32,
    relative_offset: u32,
    file_name: String,
    file_comment: String
}

impl CentralDirectoryFileHeader {

    pub fn from_reader<R>(reader: &mut R) -> Result<Self, Error>
    where R: Read + Seek {
        
        let mut cdf_bytes = vec![0; 46];
        reader.read_exact(&mut cdf_bytes)?;

        let reader_signature = LittleEndian::read_u32(&cdf_bytes[0..4]);
        assert!(reader_signature == CENTRAL_DIR_SIGNATURE);
        
        let file_name_length = LittleEndian::read_u16(&cdf_bytes[28..30]);
        let mut file_name_bytes: Vec<u8> = vec![0; file_name_length as usize];

        let extra_field_length = LittleEndian::read_u16(&cdf_bytes[30..32]);

        let file_comment_length = LittleEndian::read_u16(&cdf_bytes[32..34]);
        let mut file_comment_bytes: Vec<u8> = vec![0; file_comment_length as usize];

        let general_purpose_flag = LittleEndian::read_u16(&cdf_bytes[8..10]);

        reader.read_exact(&mut file_name_bytes)?;
        reader.seek(SeekFrom::Current(extra_field_length as i64))?;
        reader.read_exact(&mut file_comment_bytes)?;

        Ok(CentralDirectoryFileHeader {
            signature: CENTRAL_DIR_SIGNATURE,
            host_os: HostOS::from_byte(cdf_bytes[5]),
            zip_specification: ZipVersion::from_byte(cdf_bytes[4]),
            version_needed_to_extract:  ZipVersion::from_byte(cdf_bytes[6]),
            general_purpose_flag,
            file_encrypted: general_purpose_flag & 0x1 == 1,
            compression_method: CompressionMethod::from_addr(LittleEndian::read_u16(&cdf_bytes[10..12])),
            last_modified_date_time: ZipDateTime::from_addr(LittleEndian::read_u16(&cdf_bytes[14..16]), LittleEndian::read_u16(&cdf_bytes[12..14])),
            crc32: LittleEndian::read_u32(&cdf_bytes[16..20]),
            compressed_size: LittleEndian::read_u32(&cdf_bytes[20..24]),
            uncompressed_size: LittleEndian::read_u32(&cdf_bytes[24..28]),
            file_name_length,
            extra_field_length,
            file_comment_length,
            disk_number_start: LittleEndian::read_u16(&cdf_bytes[34..36]),
            internal_file_attr: LittleEndian::read_u16(&cdf_bytes[36..38]),
            external_file_attr: LittleEndian::read_u32(&cdf_bytes[38..42]),
            relative_offset: LittleEndian::read_u32(&cdf_bytes[42..46]),
            file_name: String::from_utf8(file_name_bytes).unwrap(),
            file_comment: String::from_utf8(file_comment_bytes).unwrap()
        })
    }

}

impl Into<ZipItem> for CentralDirectoryFileHeader {
    fn into(self) -> ZipItem {
        ZipItem::new(
            self.compression_method,
            self.file_name,
            self.uncompressed_size,
            self.compressed_size,
            self.last_modified_date_time,
            self.relative_offset
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    #[should_panic]
    fn central_dir_file_signature_valid() {
        let bytes = vec![0x50, 0x4B, 0x00, 0x00, 0x3F, 0x00, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x10, 0x64, 0x5C, 0x50, 0xC1, 0x5C, 0xE7, 0x5E, 0x9C, 0xEC, 0x31, 0x00, 0x39,
        0x6B, 0x33, 0x00, 0x0C, 0x00, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x48, 0x78, 0x44, 0x53, 0x65, 0x74, 0x75, 0x70,
        0x2E, 0x65, 0x78, 0x65, 0x0A, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x18, 0x00, 0x2A, 0xCD, 0x6B, 0xC3, 0x2A, 0xEE, 0xD5, 0x01, 0xAB, 0xC4, 0xEA, 0x9C, 0x2A,
        0xEE, 0xD5, 0x01, 0xAB, 0xC4, 0xEA, 0x9C, 0x2A, 0xEE, 0xD5, 0x01];

        let mut cursor = Cursor::new(bytes);
        CentralDirectoryFileHeader::from_reader(&mut cursor).unwrap();
    }

    #[test]
    fn central_directory_parsed_as_expected() {
 
        let bytes = vec![0x50, 0x4B, 0x01, 0x02, 0x3F, 0x00, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x10, 0x64, 0x5C, 0x50, 0xC1, 0x5C, 0xE7, 0x5E, 0x9C, 0xEC, 0x31, 0x00, 0x39,
        0x6B, 0x33, 0x00, 0x0C, 0x00, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x48, 0x78, 0x44, 0x53, 0x65, 0x74, 0x75, 0x70,
        0x2E, 0x65, 0x78, 0x65, 0x0A, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x18, 0x00, 0x2A, 0xCD, 0x6B, 0xC3, 0x2A, 0xEE, 0xD5, 0x01, 0xAB, 0xC4, 0xEA, 0x9C, 0x2A,
        0xEE, 0xD5, 0x01, 0xAB, 0xC4, 0xEA, 0x9C, 0x2A, 0xEE, 0xD5, 0x01];
        
        let mut cursor = Cursor::new(bytes);
        let central_dir_file = CentralDirectoryFileHeader::from_reader(&mut cursor).unwrap();

        assert_eq!(central_dir_file.host_os, HostOS::MsDos);
        assert_eq!(central_dir_file.zip_specification, ZipVersion::new(6, 3));
        assert_eq!(central_dir_file.version_needed_to_extract, ZipVersion::new(2, 0));
        assert_eq!(central_dir_file.general_purpose_flag, 0);
        assert_eq!(central_dir_file.file_encrypted, false);
        assert_eq!(central_dir_file.compression_method, CompressionMethod::Deflate);
        assert_eq!(central_dir_file.last_modified_date_time, ZipDateTime::new(28, 2, 2020, 12, 32, 32));
        assert_eq!(central_dir_file.crc32, 1592220865);
        assert_eq!(central_dir_file.compressed_size, 3271836);
        assert_eq!(central_dir_file.uncompressed_size, 3369785);
        assert_eq!(central_dir_file.file_name_length, 12);
        assert_eq!(central_dir_file.extra_field_length, 36);
        assert_eq!(central_dir_file.file_comment_length, 0);
        assert_eq!(central_dir_file.disk_number_start, 0);
        assert_eq!(central_dir_file.internal_file_attr, 0);
        assert_eq!(central_dir_file.external_file_attr, 32);
        assert_eq!(central_dir_file.relative_offset, 0);
        assert_eq!(central_dir_file.file_name, String::from("HxDSetup.exe"));
        assert_eq!(central_dir_file.file_comment, String::from(""));
    }
}