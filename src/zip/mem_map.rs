pub const FILE_HEADER_SIGNATURE: u32 = 0x04034b50;
pub const END_OF_CENTRAL_DIR_SIGNATURE: u32 = 0x06054b50;
pub const CENTRAL_DIR_SIGNATURE: u32 = 0x02014b50;

#[derive(PartialEq, Eq, Debug)]
pub enum HostOS {
    MsDos,
    Amiga,
    OpenVms,
    Unix,
    VmCms,
    AtariIst,
    OS2,
    MACINTOSH,
    ZSystem,
    CPM,
    WinNTFS,
    MVS,
    VSE,
    RISC,
    VFAT,
    AlternativeMVS,
    BEOS,
    TANDEM,
    OS400,
    OSX,
    UNUSED
}

impl HostOS {

    pub fn from_byte(byte: u8) -> Self {
        
        match byte {
             0 => HostOS::MsDos,
             1 => HostOS::Amiga,
             2 => HostOS::OpenVms,
             3 => HostOS::Unix,
             4 => HostOS::VmCms,
             5 => HostOS::AtariIst,
             6 => HostOS::OS2,
             7 => HostOS::MACINTOSH,
             8 => HostOS::ZSystem,
             9 => HostOS::CPM,
            10 => HostOS::WinNTFS,
            11 => HostOS::MVS,
            12 => HostOS::VSE,
            13 => HostOS::RISC,
            14 => HostOS::VFAT,
            15 => HostOS::AlternativeMVS,
            16 => HostOS::BEOS,
            17 => HostOS::TANDEM,
            18 => HostOS::OS400,
            19 => HostOS::OSX,
             _ => HostOS::UNUSED
        }
    }

}

#[derive(Debug, PartialEq, Eq)]
pub struct ZipVersion {
    major: u8,
    minor: u8
}

impl ZipVersion {

    pub fn from_byte(byte: u8) -> Self {

        let major = byte / 10;
        let minor = byte % 10;

        ZipVersion {
            major,
            minor
        }

    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum CompressionMethod {
    NoCompression,
    Shrunk,
    Factor1,
    Factor2,
    Factor3,
    Factor4,
    Implode,
    ReservedTokenCompression,
    Deflate,
    Deflate64,
    PKWAREDataCompressionLib,
    Reserved,
    BZIP2,
    LZMA,
    ZOSCMPSC,
    IBMTerse,
    IBMLZ77,
    Deprecated,
    ZStandard,
    MP3,
    XZ,
    JPEGVariant,
    WavPack,
    PPMd,
    Aex,
    Unknown
}

impl CompressionMethod {
    pub fn from_addr(addr: u16) -> Self {
        match addr {
            0 => CompressionMethod::NoCompression,
            1 => CompressionMethod::Shrunk,
            2 => CompressionMethod::Factor1,
            3 => CompressionMethod::Factor2,
            4 => CompressionMethod::Factor3,
            5 => CompressionMethod::Factor4,
            6 => CompressionMethod::Implode,
            7 => CompressionMethod::ReservedTokenCompression,
            8 => CompressionMethod::Deflate,
            9 => CompressionMethod::Deflate64,
            10 => CompressionMethod::PKWAREDataCompressionLib,
            11 => CompressionMethod::Reserved,
            12 => CompressionMethod::BZIP2,
            13 => CompressionMethod::Reserved,
            14 => CompressionMethod::LZMA,
            15 => CompressionMethod::Reserved,
            16 => CompressionMethod::ZOSCMPSC,
            17 => CompressionMethod::Reserved,
            18 => CompressionMethod::IBMTerse,
            19 => CompressionMethod::IBMLZ77,
            20 => CompressionMethod::Deprecated,
            93 => CompressionMethod::ZStandard,
            94 => CompressionMethod::MP3,
            95 => CompressionMethod::XZ,
            96 => CompressionMethod::JPEGVariant,
            97 => CompressionMethod::WavPack,
            98 => CompressionMethod::PPMd,
            99 => CompressionMethod::Aex,
            _ => CompressionMethod::Unknown
        }
    }
}