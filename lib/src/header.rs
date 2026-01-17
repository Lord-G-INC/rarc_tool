use binrw::prelude::*;
use binrw::Endian;

#[binrw]
#[brw(repr = u32)]
#[derive(Debug, Clone, Copy)]
// Archive Magic.
pub enum Magic {
    /// "CRAR" (Switch)
    LITTLE = 0x52415243,
    /// "RARC" (GCN, Wii)
    BIG = 0x43524152
}

impl Magic {
    /// Simple conversion to binrw::Endian.
    pub const fn to_endian(&self) -> Endian {
        match self {
            Self::BIG => Endian::Big,
            Self::LITTLE => Endian::Little
        }
    }
    /// Simple conversion from binrw::Endian.
    pub const fn from_endian(endian: Endian) -> Self {
        match endian {
            Endian::Big => Self::BIG,
            Endian::Little => Self::LITTLE
        }
    }
}


#[binrw]
#[derive(Debug, Default, Clone, Copy)]
/// The header at the start of every Archive file. Contains some basic info.
pub struct Header {
    pub size: u32,
    pub data_header_off: u32,
    pub file_data_off: u32,
    pub file_data_len: u32,
    pub mram_size: u32,
    pub aram_size: u32,
    pub dvd_size: u32
}

#[binrw]
#[derive(Debug, Default, Clone, Copy)]
/// Occurs right after [Header], contains important info for reading.
pub struct DataHeader {
    pub dir_node_count: u32,
    pub dir_node_off: u32,
    pub file_node_count: u32,
    pub file_node_off: u32,
    pub string_tbl_size: u32,
    pub string_tbl_off: u32,
    pub next_idx: u16,
    // Hack to make binrw read/write a bool, if binrw devs see this- PLEASE.
    // Make bool binrw compatible.
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |x: &bool| *x as u8)]
    #[brw(pad_after = 5)]
    pub sync: bool
}

/// Utility method to read the headers, also provides the Endian for later usage.
pub fn read_headers<R: BinReaderExt>(reader: &mut R) -> BinResult<(Endian, Header, DataHeader)> {
    let magic = Magic::read_ne(reader)?;
    let endian = magic.to_endian();
    let header: Header = reader.read_type(endian)?;
    reader.seek(std::io::SeekFrom::Start(header.data_header_off as _))?;
    let data_header = reader.read_type(endian)?;
    Ok((endian, header, data_header))
}