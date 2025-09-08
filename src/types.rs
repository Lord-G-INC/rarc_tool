use binrw::prelude::*;

#[derive(Clone, Copy, Debug, Default, BinRead, BinWrite)]
pub struct JKRArchiveHeader {
    pub filesize: u32,
    pub headersize: u32,
    pub filedataoffset: u32,
    pub filedatasize: u32,
    pub mramsize: u32,
    pub aramsize: u32,
    pub dvdsize: u32
}


#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
pub struct JKRArchiveDataHeader {
    pub dirnodecount: u32,
    pub dirnodeoffset: u32,
    pub filenodecount: u32,
    pub filenodeoffset: u32,
    pub stringtablesize: u32,
    pub stringtableoffset: u32,
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |x| *x as u8)]
    pub sync: bool
}