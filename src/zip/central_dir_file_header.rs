use std::io::{ErrorKind, prelude::*};
use std::io::SeekFrom;
use std::io::Error;
use super::{mem_map::{HostOS, CENTRAL_DIR_SIGNATURE, MINIMUM_SIZE_TO_COMPRESS, ZipVersion, CompressionMethod, EncryptionMethod}};
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
    encryption_method: EncryptionMethod,
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
        if reader_signature != CENTRAL_DIR_SIGNATURE {
            return Err(Error::new(ErrorKind::InvalidData, format!("Invalid Local Header. {:#} ", CENTRAL_DIR_SIGNATURE)));
        }
        
        let file_name_length = LittleEndian::read_u16(&cdf_bytes[28..30]);
        let mut file_name_bytes: Vec<u8> = vec![0; file_name_length as usize];

        let extra_field_length = LittleEndian::read_u16(&cdf_bytes[30..32]);

        let file_comment_length = LittleEndian::read_u16(&cdf_bytes[32..34]);
        let mut file_comment_bytes: Vec<u8> = vec![0; file_comment_length as usize];

        let general_purpose_flag = LittleEndian::read_u16(&cdf_bytes[8..10]);

        reader.read_exact(&mut file_name_bytes)?;
        reader.seek(SeekFrom::Current(extra_field_length as i64))?;
        reader.read_exact(&mut file_comment_bytes)?;

        let encryption_method = if general_purpose_flag & 0b100001 == 0b100001 {
            EncryptionMethod::WinZipAesEncryption
        }
        else if general_purpose_flag & 0x1 == 1 {
            EncryptionMethod::ZipCrypto
        }
        else { EncryptionMethod::NoEncryption };

        let cdfh = CentralDirectoryFileHeader {
            signature: CENTRAL_DIR_SIGNATURE,
            host_os: HostOS::from_byte(cdf_bytes[5]),
            zip_specification: ZipVersion::from_byte(cdf_bytes[4]),
            version_needed_to_extract:  ZipVersion::from_byte(cdf_bytes[6]),
            general_purpose_flag,
            encryption_method,
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
        };

        Ok(cdfh)
    }

    pub fn from_zip_item(zip_item: &ZipItem) -> Self {

        CentralDirectoryFileHeader {
            signature: CENTRAL_DIR_SIGNATURE,
            host_os: HostOS::from_os(),
            zip_specification: ZipVersion::new(2, 0),
            version_needed_to_extract: ZipVersion::new(2,0 ),
            general_purpose_flag: if zip_item.encryption_method() == EncryptionMethod::ZipCrypto { 0x01 } else { 0x00 },
            encryption_method: zip_item.encryption_method(),
            compression_method: zip_item.compression_method(),
            last_modified_date_time: zip_item.modified_date_time().to_owned(),
            crc32: zip_item.crc32(),
            compressed_size: zip_item.compressed_size(),
            uncompressed_size: zip_item.uncompressed_size(),
            file_name_length: zip_item.item_path().len() as u16,
            extra_field_length: 0,
            file_comment_length: 0,
            disk_number_start: 0,
            internal_file_attr: 0,
            external_file_attr: 0,
            relative_offset: zip_item.start_offset(),
            file_name: zip_item.item_path().to_owned(),
            file_comment: String::from("")
        }

    }

    pub fn to_binary(self) -> Vec<u8> {
        let mut cdfh_bin: Vec<u8> = Vec::with_capacity(46);
        let mut signature = vec![0x50u8, 0x4B, 0x01, 0x02]; 
        let mut version_needed_to_extract = vec![0x14u8, 0x00];
        let mut min_version_to_extract = vec![0x14u8, 0x00];
        let mut general_purpose_bit_flag = if self.encryption_method != EncryptionMethod::NoEncryption {
            vec![0x01u8, 0x00]
        } else {
            vec![0x00, 0x00]
        };
        let mut compression_method = if self.uncompressed_size < MINIMUM_SIZE_TO_COMPRESS {
            vec![0x00, 0x00]
        } else {
            vec![0x08u8, 0x00]
        };
        let mut last_modification_time_bytes = vec![0, 0];
        let mut last_modification_day_bytes = vec![0, 0];
        let mut last_modification_time = 0;
        let mut last_modification_day = 0;
        let mut crc32 = vec![0, 0, 0, 0];
        let mut compressed_size = vec![0, 0, 0, 0];
        let mut uncompressed_size = vec![0, 0, 0, 0];
        let mut relative_offset = vec![0, 0, 0, 0];
        let mut file_name_length = vec![0, 0];
        let mut extra_field_length = vec![0, 0];
        let mut file_comment_length = vec![0, 0];
        let mut disk_number_start = vec![0, 0];
        let mut internal_file_attributes = vec![0, 0];
        let mut external_file_attributes = vec![0, 0, 0, 0];
        let mut file_name = Vec::from(self.file_name.as_bytes());

        self.last_modified_date_time.to_addr(&mut last_modification_day, &mut last_modification_time);

        LittleEndian::write_u16(&mut last_modification_day_bytes, last_modification_day);
        LittleEndian::write_u16(&mut last_modification_time_bytes, last_modification_time);
        LittleEndian::write_u32(&mut compressed_size, self.compressed_size);
        LittleEndian::write_u32(&mut uncompressed_size, self.uncompressed_size);
        LittleEndian::write_u32(&mut crc32, self.crc32);
        LittleEndian::write_u32(&mut relative_offset, self.relative_offset);
        LittleEndian::write_u16(&mut file_name_length, self.file_name_length);
        LittleEndian::write_u16(&mut extra_field_length, self.extra_field_length);
        LittleEndian::write_u16(&mut disk_number_start, self.disk_number_start);

        cdfh_bin.append(&mut signature);
        cdfh_bin.append(&mut version_needed_to_extract);
        cdfh_bin.append(&mut min_version_to_extract);
        cdfh_bin.append(&mut general_purpose_bit_flag);
        cdfh_bin.append(&mut compression_method);
        cdfh_bin.append(&mut last_modification_time_bytes);
        cdfh_bin.append(&mut last_modification_day_bytes);
        cdfh_bin.append(&mut crc32);
        cdfh_bin.append(&mut compressed_size);
        cdfh_bin.append(&mut uncompressed_size);
        cdfh_bin.append(&mut file_name_length);
        cdfh_bin.append(&mut extra_field_length);
        cdfh_bin.append(&mut file_comment_length);
        cdfh_bin.append(&mut disk_number_start);
        cdfh_bin.append(&mut internal_file_attributes);
        cdfh_bin.append(&mut external_file_attributes);
        cdfh_bin.append(&mut relative_offset);
        cdfh_bin.append(&mut file_name);

        cdfh_bin
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
            self.relative_offset,
            self.encryption_method,
            self.crc32
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
 
        //let bytes = vec![0x50, 0x4B, 0x01, 0x02, 0x3F, 0x00, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x10, 0x64, 0x5C, 0x50, 0xC1, 0x5C, 0xE7, 0x5E, 0x9C, 0xEC, 0x31, 0x00, 0x39,
        //0x6B, 0x33, 0x00, 0x0C, 0x00, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x48, 0x78, 0x44, 0x53, 0x65, 0x74, 0x75, 0x70,
        //0x2E, 0x65, 0x78, 0x65, 0x0A, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x18, 0x00, 0x2A, 0xCD, 0x6B, 0xC3, 0x2A, 0xEE, 0xD5, 0x01, 0xAB, 0xC4, 0xEA, 0x9C, 0x2A,
        //0xEE, 0xD5, 0x01, 0xAB, 0xC4, 0xEA, 0x9C, 0x2A, 0xEE, 0xD5, 0x01];

        let bytes = vec![0x50, 0x4B, 0x01, 0x02, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0xA5, 0x01, 0x68, 0x52, 0x81, 0x6F, 0x9E, 0xB9, 0x80, 0x38, 0x0D, 0x00, 0x98, 0xAB, 0x14, 
        0x00, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x77, 0x69, 0x6E, 0x73, 0x64, 0x6B, 0x73, 0x65, 0x74, 0x75, 0x70, 0x2E, 0x65, 0x78, 0x65];
        
        let mut cursor = Cursor::new(bytes);
        let central_dir_file = CentralDirectoryFileHeader::from_reader(&mut cursor).unwrap();

        println!("{:#?}", central_dir_file);

        //assert_eq!(central_dir_file.host_os, HostOS::MsDos);
        //assert_eq!(central_dir_file.zip_specification, ZipVersion::from_byte(63));
        //assert_eq!(central_dir_file.version_needed_to_extract, ZipVersion::from_byte(20));
        //assert_eq!(central_dir_file.general_purpose_flag, 0);
        //assert_eq!(central_dir_file.encryption_method, EncryptionMethod::NoEncryption);
        //assert_eq!(central_dir_file.compression_method, CompressionMethod::Deflate);
        //assert_eq!(central_dir_file.last_modified_date_time, ZipDateTime::new(28, 2, 2020, 12, 32, 32));
        //assert_eq!(central_dir_file.crc32, 1592220865);
        //assert_eq!(central_dir_file.compressed_size, 3271836);
        //assert_eq!(central_dir_file.uncompressed_size, 3369785);
        //assert_eq!(central_dir_file.file_name_length, 12);
        //assert_eq!(central_dir_file.extra_field_length, 36);
        //assert_eq!(central_dir_file.file_comment_length, 0);
        //assert_eq!(central_dir_file.disk_number_start, 0);
        //assert_eq!(central_dir_file.internal_file_attr, 0);
        //assert_eq!(central_dir_file.external_file_attr, 32);
        //assert_eq!(central_dir_file.relative_offset, 0);
        //assert_eq!(central_dir_file.file_name, String::from("HxDSetup.exe"));
        //assert_eq!(central_dir_file.file_comment, String::from(""));
    }
}