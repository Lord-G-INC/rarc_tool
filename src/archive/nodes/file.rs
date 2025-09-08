use binrw::{prelude::*, Endian};
use super::{RcCell, move_shared};
use super::folder::FolderNode;
use crate::enums::*;
use crate::hash::calchash;

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
pub struct Node {
    pub node_idx: u16,
    pub hash: u16,
    pub attr_and_name_off: u32,
    pub data: u32,
    pub data_size: u32
}

#[derive(Debug, Default, Clone)]
pub struct FileNode {
    pub node: Node,
    pub attr: JKRFileAttr,
    pub folder: Option<RcCell<FolderNode>>,
    pub parent: Option<RcCell<FolderNode>>,
    pub name: String,
    pub name_off: u16,
    pub data: Vec<u8>
}

impl FileNode {
    pub const fn is_file(&self) -> bool {
        self.attr.contains(JKRFileAttr::FILE)
    }
    pub const fn is_folder(&self) -> bool {
        self.attr.contains(JKRFileAttr::FOLDER)
    }
    pub fn is_shortcut(&self) -> bool {
        if self.name == "." || self.name == ".." {
            self.is_folder()
        } else {
            false
        }
    }
    pub const fn preload_type(&self) -> JKRPreloadType {
        self.attr.preload_type()
    }
    pub fn read<R: BinReaderExt>(&mut self, reader: &mut R, endian: Endian) -> BinResult<()> {
        self.node = reader.read_type(endian)?;
        self.name_off = (self.node.attr_and_name_off & 0x00FFFFFF) as u16;
        self.attr = JKRFileAttr::from_bits_retain((self.node.attr_and_name_off >> 24) as u8);
        self.data.resize(self.node.data_size as _, 0);
        Ok(())
    } 
    pub fn write<W: BinWriterExt>(&self, writer: &mut W, endian: Endian) -> BinResult<()> {
        let idx = self.node.node_idx;
        writer.write_type(&idx, endian)?;
        let hash = calchash(&self.name);
        writer.write_type(&hash, endian)?;
        let mut attr = u32::from(self.attr);
        let name_off = self.name_off as u32;
        attr = (attr << 24) | name_off;
        writer.write_type(&attr, endian)?;
        let data = self.node.data;
        writer.write_type(&data, endian)?;
        let data_size = self.node.data_size;
        writer.write_type(&data_size, endian)?;
        Ok(())
    }
    pub fn create_node<A: AsRef<str>>(name: A, attr: JKRFileAttr, folder: Option<RcCell<FolderNode>>,
        parent: Option<RcCell<FolderNode>>) -> RcCell<Self> {
        let mut node = Self::default();
        node.name = String::from(name.as_ref());
        node.attr = attr;
        node.folder = folder;
        node.parent = parent.clone();
        let result = move_shared(node);
        if let Some(parent) = parent {
            let mut parent = parent.borrow_mut();
            parent.child_nodes.push(result.clone())
        }
        result
    }
}