use bitflags::bitflags;
use binrw::prelude::*;

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, BinRead, BinWrite)]
    #[br(map = |x| Self::from_bits_retain(x))]
    #[bw(map = |x: &Self| x.bits())]
    pub struct JKRFileAttr : u8 {
        const FILE = 0x01;
        const FOLDER = 0x02;
        const COMPRESSED = 0x04;
        const LOAD_TO_MRAM = 0x10;
        const LOAD_TO_ARAM = 0x20;
        const LOAD_FROM_DVD = 0x40;
        const USE_SZS = 0x80;
        const FILE_AND_COMPRESSION = 0x85;
        const FILE_AND_PRELOAD = 0x71;
    }
}

impl From<JKRFileAttr> for u32 {
    fn from(value: JKRFileAttr) -> Self {
        value.bits().into()
    }
}

impl From<u32> for JKRFileAttr {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value as u8)
    }
}

impl JKRFileAttr {
    pub const fn preload_type(&self) -> JKRPreloadType {
        if self.contains(Self::LOAD_TO_MRAM) {
            JKRPreloadType::MRAM
        } else if self.contains(Self::LOAD_TO_ARAM) {
            JKRPreloadType::ARAM
        } else if self.contains(Self::LOAD_FROM_DVD) {
            JKRPreloadType::DVD
        } else {
            JKRPreloadType::NONE
        }
    }
}

#[repr(i8)]
pub enum JKRPreloadType {
    NONE = -1,
    MRAM = 0,
    ARAM,
    DVD
}