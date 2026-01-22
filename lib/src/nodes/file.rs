use binrw::prelude::*;
use bitflags::bitflags;
use std::ops::{Deref, DerefMut};
use crate::make_reference;

use super::Reference;
use super::directory::Directory;
use std::path::PathBuf;

#[binrw]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// A File's Attributes. Uses bitflags for the actual variants.
pub struct FileAttr(pub u8);

bitflags! {
    impl FileAttr : u8 {
        /// Is a file.
        const FILE = 0x1;
        /// Is a folder.
        const FOLDER = 0x2;
        /// Is compressed in some way.
        const COMPRESSED = 0x4;
        /// Load this to the Main RAM (Default usually).
        const LOAD_TO_MRAM = 0x10;
        /// Load this to the Auxiliary RAM (GameCube only).
        const LOAD_TO_ARAM = 0x20;
        /// Priority load straight from the DVD when needed.
        const LOAD_FROM_DVD = 0x40;
        /// This is Yaz0 compressed, [FileAttr::COMPRESSED] should also be on.
        const USE_SZS = 0x80;
        /// Combo of FILE and COMPRESSED.
        const FILE_AND_COMPRESSION = 0x85;
        /// Combo of FILE and MRAM.
        const FILE_AND_PRELOAD = 0x71;
    }
}

impl Deref for FileAttr {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FileAttr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[binrw]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// A File's actual information.
pub struct Node {
    pub id: u16,
    pub hash: u16,
    pub attr_and_off: u32,
    pub data: u32,
    pub data_size: u32
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// A File within a Archive. Can point to a folder and have a parent.
pub struct File {
    pub node: Node,
    pub attr: FileAttr,
    pub folder: Option<Reference<Directory>>,
    pub parent: Option<Reference<Directory>>,
    pub name: String,
    pub name_off: u16,
    pub data: Vec<u8>
}

impl File {
    /// Utility to check if this is a file.
    pub const fn is_file(&self) -> bool {
        self.attr.contains(FileAttr::FILE)
    }
    /// Utility to check if this is a directory.
    pub const fn is_dir(&self) -> bool {
        self.attr.contains(FileAttr::FOLDER)
    }
    /// Checks if this is a dir and the name is "." or ".."
    pub fn is_shortcut(&self) -> bool {
        if self.name == "." || self.name == ".." {
            self.is_dir()
        } else {
            false
        }
    }
    /// Read this File based off the endian.
    pub fn read<R: BinReaderExt>(&mut self, reader: &mut R, endian: binrw::Endian) -> BinResult<()> {
        self.node = reader.read_type(endian)?; 
        self.name_off = (self.node.attr_and_off & 0x00FFFFFF) as u16;
        self.attr = FileAttr((self.node.attr_and_off >> 24) as u8); 
        Ok(())
    }
    /// Write this File based off the endian. 
    /// **NOTE: DOES NOT WRITE THE ACTUAL FILE DATA. THIS ONLY WRITES THE**
    /// **NODE DATA.**
    pub fn write<W: BinWriterExt>(&self, writer: &mut W, endian: binrw::Endian) -> BinResult<()> {
        writer.write_type(&self.node.id, endian)?;
        writer.write_type(&super::calc_hash(&self.name), endian)?;
        let attr = self.attr.0 as u32;
        let off = self.name_off as u32;
        let total = (attr << 24) | off;
        writer.write_type(&total, endian)?;
        writer.write_type(&self.node.data, endian)?;
        if self.is_file() {
            writer.write_type(&self.node.data_size, endian)?;
        } else if self.is_dir() {
            writer.write_type(&0x10u32, endian)?;
        }
        Ok(())
    }
    /// Ditto of [Directory::to_string].
    pub fn to_string(&self) -> String {
        let mut names = vec![];
        names.push(self.name.clone());
        if let Some(parent) = &self.parent {
            parent.borrow().add_name(&mut names);
        }
        names.reverse();
        let mut result = PathBuf::new();
        for name in names {
            result = result.join(name);
        }
        result.to_string_lossy().into()
    }
    
    pub fn create(name: &str, attr: FileAttr, folder: Option<Reference<Directory>>, parent: Option<Reference<Directory>>) 
     -> Reference<File> {
        let file = File {name: name.into(), attr, folder: folder.clone(), parent: parent.clone(), ..Default::default()};
        let result = make_reference(file);
        if let Some(parent) = &parent {
            parent.borrow_mut().children.push(result.clone());
        }
        result
    }
}
