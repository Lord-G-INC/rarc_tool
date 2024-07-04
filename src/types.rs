use binrw::prelude::*;
use std::io::prelude::*;

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


#[derive(Debug, Default, Clone, Copy)]
pub struct JKRArchiveDataHeader {
    pub dirnodecount: u32,
    pub dirnodeoffset: u32,
    pub filenodecount: u32,
    pub filenodeoffset: u32,
    pub stringtablesize: u32,
    pub stringtableoffset: u32,
    pub sync: bool
}

impl BinRead for JKRArchiveDataHeader {
    type Args<'a> = ();
    fn read_options<R: Read + Seek>(
            reader: &mut R,
            endian: binrw::Endian,
            _: Self::Args<'_>,
        ) -> BinResult<Self> {
        let mut result = Self::default();
        let Self {dirnodecount, dirnodeoffset, filenodeoffset,
        filenodecount, stringtablesize, stringtableoffset, sync}
        = &mut result;
        *dirnodecount = reader.read_type(endian)?;
        *dirnodeoffset = reader.read_type(endian)?;
        *filenodecount = reader.read_type(endian)?;
        *filenodeoffset = reader.read_type(endian)?;
        *stringtablesize = reader.read_type(endian)?;
        *stringtableoffset = reader.read_type(endian)?;
        *sync = reader.read_ne::<u8>()? != 0;
        Ok(result)
    }
}

impl BinWrite for JKRArchiveDataHeader {
    type Args<'a> = ();
    fn write_options<W: Write + Seek>(
            &self,
            writer: &mut W,
            endian: binrw::Endian,
            _: Self::Args<'_>,
        ) -> BinResult<()> {
        let Self {dirnodecount, dirnodeoffset, filenodeoffset, filenodecount,
        stringtablesize, stringtableoffset, ..} = self;
        writer.write_type(dirnodecount, endian)?;
        writer.write_type(dirnodeoffset, endian)?;
        writer.write_type(filenodecount, endian)?;
        writer.write_type(filenodeoffset, endian)?;
        writer.write_type(stringtablesize, endian)?;
        writer.write_type(stringtableoffset, endian)?;
        writer.write_ne(&(self.sync as u8))?;
        Ok(())
    }
}