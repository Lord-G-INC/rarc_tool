use std::collections::{BTreeMap, HashMap};
use std::ops::*;
use super::archive::Archive;

use binrw::{NullString, prelude::*};

#[derive(Debug, Default, Clone)]
pub struct Table {
    pub table: BTreeMap<u32, String>
}

impl Table {
    pub fn lookup(&self) -> HashMap<String, u32> {
        let mut result = HashMap::new();
        for (key, str) in &self.table {
            result.insert(str.clone(), *key);
        }
        result
    }
    pub fn write<W: BinWriterExt>(&self, writer: &mut W) -> BinResult<()> {
        for (_, str) in &self.table {
            let string = NullString::from(str.as_str());
            string.write_ne(writer)?;
        }
        let offset = writer.stream_position()?.next_multiple_of(32);
        while writer.stream_position()? < offset {
            0u8.write_ne(writer)?;
        }
        Ok(())
    }
    
    pub fn total_size(&self) -> u32 {
        let mut total = 0u32;
        for (_, str) in &self.table {
            total += str.len() as u32 + 1;
        }
        total.next_multiple_of(32)
    }

    pub fn add<A: AsRef<str>>(&mut self, item: A) -> u32 {
        let item = String::from(item.as_ref());
        if let Some(off) = self.lookup().get(&item) {
            *off
        } else {
            if let Some((&loff, lstr)) = self.table.iter().last() {
                let mut off = loff;
                off += (lstr.len() + 1) as u32;
                self.table.insert(off, item);
                off
            } else {
                self.table.insert(0, item);
                0
            }
        }
    }
}

impl Deref for Table {
    type Target = BTreeMap<u32, String>;
    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

impl DerefMut for Table {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.table
    }
}

impl Archive {
    pub fn read_table<R: BinReaderExt>(&mut self, reader: &mut R) -> BinResult<Table> {
        let mut result = Table::default();
        let current = reader.stream_position()?;
        let offset = self.data_header.string_tbl_off + self.header.data_header_off;
        let start = reader.seek(std::io::SeekFrom::Start(offset as _))?;
        let end = start + self.data_header.string_tbl_size as u64;
        let mut off = 0u32;
        while reader.stream_position()? < end {
            let ne = NullString::read_ne(reader)?;
            let str = String::from_utf8(ne.0).unwrap();
            let len = str.len() as u32;
            if str.is_empty() {break;}
            result.table.insert(off, str);
            off += len + 1;
        }
        reader.seek(std::io::SeekFrom::Start(current))?;
        Ok(result)
    }
}