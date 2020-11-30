use byteorder::{ByteOrder, LittleEndian};

pub struct DataDescriptor {
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32
}

impl DataDescriptor {

    pub fn from_bytes(bytes: &[u8]) -> Self {

        DataDescriptor {
            crc32: LittleEndian::read_u32(&bytes[0..4]),
            compressed_size: LittleEndian::read_u32(&bytes[4..8]),
            uncompressed_size: LittleEndian::read_u32(&bytes[8..12])
        }
    }

}