use std::{borrow::Cow, io::{Cursor, Read}, path::PathBuf};

use binrw::prelude::*;
use binrw::*;
use bitflags::*;
use crate::seektask;

bitflags! {
    #[derive(Default)]
    #[binrw]
    pub struct FileAttr: u8 {
        const FILE = 0x1;
        const FOLDER = 0x2;
        const COMPRESSED = 0x4;
        const LOADTOMRAM = 0x10;
        const LOADTOARAM = 0x20;
        const LOADFROMDVD = 0x40;
        const USESZS = 0x80;
        const FILEANDCOMPRESSION = 0x85;
        const FILEANDPRELOAD = 0x71;
    }
}

bitflags! {
    #[derive(Default)]
    #[binrw]
    pub struct PreloadType: i8 {
        const NONE = -1;
        const MRAM = 0;
        const ARAM = 1;
        const DVD = 2;
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[binrw]
pub struct Header {
    pub filesize: u32,
    pub headersize: u32,
    pub filedataoff: u32,
    pub filedatasize: u32,
    pub mramsize: u32,
    pub aramsize: u32,
    pub dvdsize: u32
}

#[derive(Debug, Clone, Copy, Default)]
#[binrw]
pub struct DataHeader {
    pub dirnodecount: u32,
    pub dirnodeoff: u32,
    pub filenodecount: u32,
    pub filenodeoff: u32,
    pub stringtablesize: u32,
    pub stringtableoff: u32
}

pub mod folder {
    use super::*;
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    #[binrw]
    pub struct Node {
        pub shortname: [u8; 4],
        pub nameoff: u32,
        pub hash: u16,
        pub filecount: u16,
        pub firstfileoff: u32
    }
}

pub mod dir {
    use super::*;
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    #[binrw]
    pub struct Node {
        pub nodeidx: u16,
        pub hash: u16,
        pub attrandnameoff: u32,
        pub data: u32,
        pub datasize: u32
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileNode {
    pub node: folder::Node,
    pub isroot: bool,
    pub name: String,
    pub dir: Option<*const DirNode>
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DirNode {
    pub node: dir::Node,
    pub attr: FileAttr,
    pub name: String,
    pub nameoff: u16,
    pub data: Vec<u8>,
    pub folder: Option<*const FileNode>,
    pub parent: Option<*const FileNode>
}

#[derive(Debug, Clone)]
pub struct RARC<'a> {
    pub folders: Vec<FileNode>,
    pub dirs: Vec<DirNode>,
    pub header: Header,
    pub dataheader: DataHeader,
    pub nextidx: u16,
    pub sync: bool,
    pub data: Cow<'a, [u8]>,
    pub endian: Endian
}

impl<'a> Default for RARC<'a> {
    fn default() -> Self {
        Self 
        { folders: Default::default(), dirs: Default::default(), header: Default::default(),
            dataheader: Default::default(), nextidx: Default::default(), sync: Default::default(),
            data: Default::default(), endian: Endian::NATIVE 
        }
    }
}

#[inline(always)]
fn read<T: BinRead, R: BinReaderExt>(endian: Endian, reader: &mut R) -> binrw::BinResult<T> where <T as binrw::BinRead>::Args: std::default::Default {
    Ok(match endian {
        Endian::Big => reader.read_be()?,
        Endian::Little => reader.read_le()?,
    })
}

impl <'a> RARC<'a> {
    pub fn read<T: Into<Cow<'a, [u8]>>>(data: T) -> Self {
        let data: Cow<'a, [u8]> = data.into();
        let mut reader = Cursor::new(data.as_ref());
        reader.set_position(4);
        let magic = seektask::readntstringat(&mut reader, 0);
        let endian = match magic.as_str() {
            "RARC" => Endian::Big,
            "CRAR" => Endian::Little,
            _ => Endian::NATIVE
        };
        let header: Header = read(endian, &mut reader).unwrap_or_default();
        let dataheader: DataHeader = read(endian, &mut reader).unwrap_or_default();
        let nextidx: u16 = read(endian, &mut reader).unwrap_or_default();
        let sync = u8::read(&mut reader).unwrap_or_default() != 0;
        reader.set_position((dataheader.dirnodeoff + header.headersize) as u64);
        let mut folders = Vec::<FileNode>::with_capacity(dataheader.dirnodecount as usize);
        let mut dirs = Vec::<DirNode>::with_capacity(dataheader.filenodecount as usize);
        for i in 0..folders.capacity() {
            let mut node: FileNode = Default::default();
            node.node = read(endian, &mut reader).unwrap_or_default();
            let pos = (dataheader.stringtableoff + header.headersize + node.node.nameoff) as u64;
            node.name = seektask::readntstringat(&mut reader, pos);
            if i == 0 {
                node.isroot = true;
            }
            folders.push(node);
        }
        reader.set_position((dataheader.filenodeoff + header.headersize) as u64);
        for _ in 0..dirs.capacity() {
            let mut node: DirNode = Default::default();
            node.node = read(endian, &mut reader).unwrap_or_default();
            reader.set_position(reader.position() + 4);
            node.nameoff = (node.node.attrandnameoff & 0x00FFFFFF) as u16;
            node.attr = FileAttr { bits: (node.node.attrandnameoff >> 24) as u8 };
            let pos = (dataheader.stringtableoff + header.headersize + node.nameoff as u32) as u64;
            node.name = seektask::readntstringat(&mut reader, pos);
            if node.attr.contains(FileAttr::FILE) {
                let pos = (header.filedataoff + header.headersize + node.node.data) as u64;
                seektask::seektask(&mut reader, pos, |task| {
                    node.data = vec![0u8; node.node.datasize as usize];
                    task.reader.read_exact(&mut node.data).unwrap();
                })
            }
            dirs.push(node);
        }
        let mut res = Self {
            folders,
            dirs,
            data,
            dataheader,
            header,
            sync,
            nextidx,
            endian
        };
        res.sortparents();
        res
    }
    fn sortparents(&mut self) {
        for dir in &mut self.dirs {
            if dir.attr.contains(FileAttr::FOLDER) && dir.node.data != u32::MAX {
                dir.folder = Some(&self.folders[dir.node.data as usize]);
                if dir.node.hash == self.folders[dir.node.data as usize].node.hash {
                    self.folders[dir.node.data as usize].dir = Some(dir);
                }
            }
        }
        for folder in &self.folders {
            for y in folder.node.firstfileoff..(folder.node.firstfileoff+folder.node.filecount as u32) {
                let y = y as usize;
                let dir = &mut self.dirs[y];
                dir.parent = Some(folder);
            }
        }
    }
    fn getchildren(&self, node: &FileNode) -> Vec<&DirNode> {
        let mut idxs = vec![];
        for y in node.node.firstfileoff..(node.node.firstfileoff+node.node.filecount as u32) {
            idxs.push(y as usize);
        }
        self.dirs.iter().enumerate().filter(|(x, _)| idxs.contains(x)).map(|(_, x)| x)
        .collect()
    }
    fn findfolder(&self, dir: &DirNode) -> Option<&FileNode> {
        match dir.folder {
            Some(n) => unsafe { n.as_ref() },
            None => None
        }
    }
    fn getroot(&self, dirs: &Vec<&DirNode>) -> Vec<&FileNode> {
        let mut result = vec![];
        let mut dirs = dirs.clone();
        let mut fnode = dirs[dirs.len() - 2];
        while let Some(folder) = self.findfolder(fnode) {
            if !folder.isroot {
                result.push(folder);
                dirs = self.getchildren(folder);
                fnode = dirs[dirs.len() - 1];
                continue;
            } else {
                result.push(folder);
                break;
            }
        }
        result.reverse();
        result
    }
    pub fn extract(&self) {
        for folder in &self.folders {
            let children = self.getchildren(folder);
            let tree = self.getroot(&children);
            let mut path = PathBuf::from(tree[0].name.clone());
            for i in 1..tree.len() {
                path = path.join(&tree[i].name);
            }
            for child in children.into_iter()
            .filter(|x| x.attr.contains(FileAttr::FILE)) {
                std::fs::create_dir_all(&path).unwrap();
                std::fs::write(path.join(&child.name), &child.data).unwrap();
            }
        }
    }
}