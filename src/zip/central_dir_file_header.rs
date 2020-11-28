use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::Error;
use super::mem_map::{HostOS, CENTRAL_DIR_SIGNATURE, ZipVersion, CompressionMethod};
use byteorder::{LittleEndian, ByteOrder};
use super::date_time::*;
pub const MIN_CENTRAL_DIRECTORY_SIZE: usize = 46;

#[derive(Debug)]
pub struct CentralDirectoryFileHeader {
    signature: u32,
    host_os: HostOS,
    zip_specification: ZipVersion,
    version_needed_to_extract: ZipVersion,
    general_purpose_flag: u16,
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
        
        let mut signature_bytes = vec![0; 4];
        reader.read_exact(&mut signature_bytes)?;
        let reader_signature = LittleEndian::read_u32(&signature_bytes);

        assert!(reader_signature == CENTRAL_DIR_SIGNATURE);
        reader.seek(SeekFrom::Current(-4))?;


        let mut cdf_bytes = vec![0; MIN_CENTRAL_DIRECTORY_SIZE];
        println!("{:?}", reader.seek(SeekFrom::Current(0)).unwrap());
        reader.read_exact(&mut cdf_bytes)?;
        
        let file_name_length = LittleEndian::read_u16(&cdf_bytes[28..30]);
        let mut file_name_bytes: Vec<u8> = vec![0; file_name_length as usize];

        let extra_field_length = LittleEndian::read_u16(&cdf_bytes[30..32]);

        let file_comment_length = LittleEndian::read_u16(&cdf_bytes[32..34]);
        let mut file_comment_bytes: Vec<u8> = vec![0; file_comment_length as usize];

        println!("{:?}", reader.seek(SeekFrom::Current(0)).unwrap());
        reader.read_exact(&mut file_name_bytes)?;
        reader.seek(SeekFrom::Current(extra_field_length as i64))?;
        reader.read_exact(&mut file_comment_bytes)?;

        Ok(CentralDirectoryFileHeader {
            signature: CENTRAL_DIR_SIGNATURE,
            host_os: HostOS::from_byte(cdf_bytes[5]),
            zip_specification: ZipVersion::from_byte(cdf_bytes[4]),
            version_needed_to_extract:  ZipVersion::from_byte(cdf_bytes[6]),
            general_purpose_flag: LittleEndian::read_u16(&cdf_bytes[8..10]),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use super::super::eof_central_dir;
    use super::super::zip_metadata;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_central_directory() {
 
        let file = File::open(r"C:\eula.zip").unwrap();
        let mut buf_reader = BufReader::new(file);
        buf_reader.seek(SeekFrom::Start(35703)).unwrap();

        let central_dir_file = CentralDirectoryFileHeader::from_reader(&mut buf_reader);
        println!("{:?}", central_dir_file.unwrap());
    }
}