use binrw::prelude::*;

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
pub struct FileHeader {
    pub file_size: u32,
    pub header_size: u32,
    pub data_off: u32,
    pub data_size: u32,
    pub mram_size: u32,
    pub aram_size: u32,
    pub dvd_size: u32
}

impl FileHeader {
    pub const fn data_offset(&self) -> u64 {
        (self.data_off + self.header_size) as u64
    }
}

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
pub struct DataHeader {
    pub dir_node_count: u32,
    pub dir_node_off: u32,
    pub file_node_count: u32,
    pub file_node_off: u32,
    pub string_table_size: u32,
    pub string_table_off: u32,
    pub next_idx: u16,
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |x: &bool| *x as u8)]
    pub sync: bool
}

impl DataHeader {
    pub const fn dir_offset(&self, header: &FileHeader) -> u64 {
        (self.dir_node_off + header.header_size) as u64
    }
    pub const fn file_offset(&self, header: &FileHeader) -> u64 {
        (self.file_node_off + header.header_size) as u64
    }
    pub const fn string_offset(&self, header: &FileHeader) -> u64 {
        (self.string_table_off + header.header_size) as u64
    }
}