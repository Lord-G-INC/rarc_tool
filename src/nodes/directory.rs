use std::path::{Path, PathBuf};

use binrw::prelude::*;
use super::Reference;
use super::file::File;

#[binrw]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// The directory's actual information.
pub struct Node {
    pub short_name: [u8; 4],
    pub name_off: u32,
    pub hash: u16,
    pub file_count: u16,
    pub file_off: u32
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// A Directory within an Archive. May point to a File or contain child file(s).
pub struct Directory {
    pub node: Node,
    pub is_root: bool,
    pub name: String,
    pub file: Option<Reference<File>>,
    pub children: Vec<Reference<File>>
}

impl Directory {
    /// Generates this Directory's short name, always a length of 4.
    pub const fn short_name(&self) -> [u8; 4] {
        if self.is_root {
            return [b'R', b'O', b'O', b'T'];
        }
        let mut result = [b' '; 4];
        let bytes = self.name.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if i >= 4 {break;}
            result[i] = bytes[i].to_ascii_uppercase();
            i += 1;
        }
        result
    }
    /// Reads this Directory with the given endian.
    pub fn read<R: BinReaderExt>(&mut self, reader: &mut R, endian: binrw::Endian) -> BinResult<()> {
        self.node = reader.read_type(endian)?;
        Ok(())
    }
    /// Writes this Directory with the given endian.
    pub fn write<W: BinWriterExt>(&self, writer: &mut W, endian: binrw::Endian) -> BinResult<()> {
        writer.write(&self.short_name())?;
        self.node.name_off.write_options(writer, endian, ())?;
        let hash = super::calc_hash(&self.name);
        hash.write_options(writer, endian, ())?;
        let count = self.children.len() as u16;
        count.write_options(writer, endian, ())?;
        self.node.file_off.write_options(writer, endian, ())?;
        Ok(()) 
    }
    /// to_string is a bit of a bad name, what this really does is generate
    /// a fullpath name pointing all the way to the root.
    pub fn to_string(&self) -> String {
        let mut names = vec![];
        self.add_name(&mut names);
        names.reverse();
        let mut result = PathBuf::new();
        for name in names {
            result = result.join(name);
        }
        result.to_string_lossy().into()
    }
    /// Add this Directory's name, then attempts to add the Parent (if it exists).
    pub(crate) fn add_name(&self, names: &mut Vec<String>) {
        names.push(self.name.clone());
        if let Some(file) = &self.file {
            if let Some(dir) = file.borrow().parent.as_ref() {
                dir.borrow().add_name(names);
            }
        }
    }
    /// Unpack this Directory and **all** children.
    pub fn unpack<A: AsRef<Path>>(&self, dir: A) -> std::io::Result<()> {
        let dir = dir.as_ref();
        if self.is_root {
            std::fs::create_dir_all(dir.join(&self.name))?;
        }
        std::fs::create_dir_all(dir)?;
        for c in &self.children {
            let child = c.borrow();
            if child.name == "." || child.name == ".." {
                continue;
            }
            let dirname = dir.to_string_lossy();
            let fullname = match dirname.contains(&self.name) {
                true => dir.join(&child.name),
                false => dir.join(&self.name).join(&child.name)
            };
            if child.is_dir() && let Some(dir) = 
                &child.folder {
                dir.borrow().unpack(fullname)?;
            } else if child.is_file() {
                std::fs::write(fullname, &child.data)?;
            }
        }
        Ok(())
    }
}