pub mod header;
pub mod nodes;

use binrw::{prelude::*, Endian, NullString};
use std::io::SeekFrom;
use std::path::Path;

pub use nodes::*;
pub use header::*;

#[derive(Debug, Default, Clone)]
pub struct JKRArchive {
    pub header: FileHeader,
    pub data_header: DataHeader,
    pub folders: Vec<RcCell<FolderNode>>,
    pub files: Vec<RcCell<FileNode>>,
    pub root: RcCell<FolderNode>
}

impl JKRArchive {
    pub const BIG : u32 = 1129464146;
    pub const LITTLE : u32 = 1380012611;
    pub fn sync(&mut self) -> &mut bool {
        &mut self.data_header.sync
    }
    pub fn next_idx(&mut self) -> &mut u16 {
        &mut self.data_header.next_idx
    }
    pub fn read<R: BinReaderExt>(&mut self, reader: &mut R) -> BinResult<()> {
        let magic = reader.read_ne()?;
        let endian = match magic {
            Self::BIG => Endian::Big,
            Self::LITTLE => Endian::Little,
            _ => Endian::NATIVE
        };
        self.header = reader.read_type(endian)?;
        self.data_header = reader.read_type(endian)?;
        let d_off = self.data_header.dir_offset(&self.header);
        let f_off = self.data_header.file_offset(&self.header);
        let s_off = self.data_header.string_offset(&self.header);
        let fd_off = self.header.data_offset();
        self.folders.reserve(self.data_header.dir_node_count as _);
        self.files.reserve(self.data_header.file_node_count as _);
        reader.seek(SeekFrom::Start(d_off))?;
        for i in 0..self.data_header.dir_node_count {
            let mut node = FolderNode::default();
            node.read(reader, endian)?;
            let off = s_off + node.node.name_off as u64;
            let pos = reader.stream_position()?;
            reader.seek(SeekFrom::Start(off))?;
            let str : NullString = reader.read_ne()?;
            node.name = str.try_into().unwrap();
            reader.seek(SeekFrom::Start(pos))?;
            if i == 0 {
                node.root = true;
                self.root = move_shared(node);
                self.folders.push(self.root.clone());
            } else {
                self.folders.push(move_shared(node));
            }
        }
        reader.seek(SeekFrom::Start(f_off))?;
        for _ in 0..self.data_header.file_node_count {
            let mut node = FileNode::default();
            node.read(reader, endian)?;
            reader.seek_relative(4)?;
            let off = s_off + node.name_off as u64;
            let mut pos = reader.stream_position()?;
            reader.seek(SeekFrom::Start(off))?;
            let str : NullString = reader.read_ne()?;
            node.name = str.try_into().unwrap();
            reader.seek(SeekFrom::Start(pos))?;
            if node.is_folder() && node.node.data != u32::MAX {
                node.folder = Some(self.folders[node.node.data as usize].clone());
            } else if node.is_file() {
                let off = fd_off + node.node.data as u64;
                pos = reader.stream_position()?;
                reader.seek(SeekFrom::Start(off))?;
                reader.read(&mut node.data)?;
                reader.seek(SeekFrom::Start(pos))?;
            }
            self.files.push(move_shared(node));
        }
        for i in 0..self.files.len() {
            let file = &mut self.files[i];
            let mut node = file.borrow_mut();
            let hash = node.node.hash;
            if node.is_folder() {
                if let Some(folder) = &mut node.folder {
                    let mut folder = folder.borrow_mut();
                    if folder.node.hash == hash {
                        folder.file = Some(file.clone());
                    }
                }
            }
        }
        for i in 0..self.folders.len() {
            let folder = &mut self.folders[i];
            let mut node = folder.borrow_mut();
            let off = node.node.first_file_off as usize;
            let count = off + node.node.file_count as usize;
            for y in off..count {
                let file = &self.files[y];
                let mut fnode = file.borrow_mut();
                fnode.parent = Some(folder.clone());
                node.child_nodes.push(file.clone());
            }
        }
        Ok(())
    }
    pub fn unpack<A: AsRef<Path>>(&self, path: A) -> BinResult<()> {
        self.root.borrow().unpack(path)
    }
}