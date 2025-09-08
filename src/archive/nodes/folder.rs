use binrw::{prelude::*, Endian};
use super::file::FileNode;
use super::RcCell;
use crate::hash::calchash;
use std::path::Path;

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
pub struct Node {
    pub short_name: [u8; 4],
    pub name_off: u32,
    pub hash: u16,
    pub file_count: u16,
    pub first_file_off: u32
}

#[derive(Debug, Default, Clone)]
pub struct FolderNode {
    pub node: Node,
    pub root: bool,
    pub name: String,
    pub file: Option<RcCell<FileNode>>,
    pub child_nodes: Vec<RcCell<FileNode>>
}

impl FolderNode {
    pub fn read<R: BinReaderExt>(&mut self, reader: &mut R, endian: Endian) -> BinResult<()> {
        self.node = reader.read_type(endian)?;
        self.child_nodes.reserve(self.node.file_count as _);
        Ok(())
    }
    pub fn short_name(&self) -> String {
        if self.root {
            return "ROOT".into();
        }
        let mut ret = self.name.clone();
        while ret.len() < 4 {
            ret.push(' ');
        }
        ret.to_uppercase()
    }
    pub fn write<W: BinWriterExt>(&self, writer: &mut W, endian: Endian) -> BinResult<()> {
        writer.write(self.short_name().as_bytes())?;
        let hash = calchash(&self.name);
        let size = self.child_nodes.len() as u16;
        writer.write_type(&hash, endian)?;
        writer.write_type(&size, endian)?;
        writer.write_type(&self.node.first_file_off, endian)?;
        Ok(())
    }
    pub fn unpack<A: AsRef<Path>>(&self, path: A) -> BinResult<()> {
        let path = path.as_ref();
        if !path.is_dir() {
            return Ok(());
        }
        if self.root {
            std::fs::create_dir_all(path.join(&self.name))?;
        }
        std::fs::create_dir_all(path)?;
        for child in &self.child_nodes {
            let node = child.borrow();
            if node.name == "." || node.name == ".." {
                continue;
            }
            let path_name = path.to_string_lossy().to_string();
            let fullname = match path_name.contains(&self.name) {
                true => path.join(&node.name),
                false => path.join(&self.name).join(&node.name)
            };
            if node.is_folder() {
                std::fs::create_dir_all(&fullname)?;
                if let Some(folder) = &node.folder {
                    folder.borrow().unpack(fullname)?;
                }
            } else if node.is_file() {
                std::fs::write(fullname, &node.data)?;
            }
        }
        Ok(())
    }
}