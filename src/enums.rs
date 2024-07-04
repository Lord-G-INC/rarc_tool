use bitflags::bitflags;
use binrw::prelude::*;

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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

impl BinRead for JKRFileAttr {
    type Args<'a> = ();
    fn read_options<R: std::io::Read + std::io::Seek>(
            reader: &mut R,
            _: binrw::Endian,
            _: Self::Args<'_>,
        ) -> BinResult<Self> {
        Ok(JKRFileAttr::from_bits_retain(reader.read_ne()?))
    }
}

impl BinWrite for JKRFileAttr {
    type Args<'a> = ();
    fn write_options<W: std::io::Write + std::io::Seek>(
            &self,
            writer: &mut W,
            _: binrw::Endian,
            _: Self::Args<'_>,
        ) -> BinResult<()> {
        writer.write_ne(&self.0.0)
    }
}

impl From<JKRFileAttr> for u32 {
    fn from(value: JKRFileAttr) -> Self {
        value.0.0 as u32
    }
}

impl From<u32> for JKRFileAttr {
    fn from(value: u32) -> Self {
        Self::from_bits_truncate(value as u8)
    }
}

#[repr(i8)]
pub enum JKRPreloadType {
    NONE = -1,
    MRAM = 0,
    ARAM,
    DVD
}