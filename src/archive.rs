use std::io::SeekFrom::{Start, Current};
use std::path::{Path, PathBuf};

use binrw::prelude::*;
use binrw::Endian;
use crate::types::*;
use crate::enums::*;
use crate::string_table::StringTable;
use crate::seek_task::SeekTask;
use crate::hash::calchash;


#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
pub struct JKRFolderInfo {
    pub shortname: [u8; 4],
    pub nameoffs: u32,
    pub hash: u16,
    pub filecount: u16,
    pub firstfileoff: u32
}

#[derive(Debug, Default, Clone, BinRead, BinWrite)]
pub struct JKRFolderNode {
    pub node: JKRFolderInfo,
    #[brw(ignore)]
    pub isroot: bool,
    #[brw(ignore)]
    pub name: String,
    
    #[brw(ignore)]
    pub filenode: Option<usize>,
    #[br(calc(Vec::with_capacity(node.filecount as usize)))]
    #[bw(ignore)]
    pub childnodes: Vec<usize>
}

impl JKRFolderNode {
    pub fn get_short_name(&self) -> String {
        if self.isroot {
            return String::from("ROOT");
        }
        let mut name = self.name.clone();
        while name.len() < 4 {
            name.push(' ');
        }
        name[4..].to_string().to_uppercase()
    }
}

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite, PartialEq)]
pub struct JKRFileInfo {
    pub nodeidx: u16,
    pub hash: u16,
    pub attrandnameoffs: u32,
    pub data: u32,
    pub datasize: u32
}

#[derive(Debug, Default, Clone, BinRead, PartialEq)]
pub struct JKRFileNode {
    pub node: JKRFileInfo,
    #[br(ignore)]
    pub attr: JKRFileAttr,
    #[br(ignore)]
    pub nameoffs: u16,

    #[br(ignore)]
    pub name: String,
    #[br(ignore)]
    pub foldernode: Option<usize>,
    #[br(ignore)]
    pub parentnode: Option<usize>,
    #[br(ignore)]
    pub data: Vec<u8>
}

impl BinWrite for JKRFileNode {
    type Args<'a> = ();
    fn write_options<W: std::io::Write + std::io::Seek>(
            &self,
            writer: &mut W,
            endian: Endian,
            _: Self::Args<'_>,
        ) -> BinResult<()> {
        writer.write_type(&self.node.nodeidx, endian)?;
        writer.write_type(&calchash(&self.name), endian)?;
        let attr = u32::from(self.attr);
        let nameoff = self.nameoffs as u32;
        let full = attr << 24 | nameoff;
        writer.write_type(&full, endian)?;
        writer.write_type(&self.node.data, endian)?;
        writer.write_type(&self.node.datasize, endian)
    }
}

impl JKRFileNode {
    pub const fn preload_type(&self) -> JKRPreloadType {
        if self.attr.contains(JKRFileAttr::LOAD_TO_MRAM) {
            JKRPreloadType::MRAM
        } else if self.attr.contains(JKRFileAttr::LOAD_TO_ARAM) {
            JKRPreloadType::ARAM
        } else if self.attr.contains(JKRFileAttr::LOAD_FROM_DVD) {
            JKRPreloadType::DVD
        } else {
            JKRPreloadType::NONE
        }
    }
    pub const fn is_dir(&self) -> bool {
        self.attr.contains(JKRFileAttr::FOLDER)
    }
    pub const fn is_file(&self) -> bool {
        self.attr.contains(JKRFileAttr::FILE)
    }
    pub fn is_shortcut(&self) -> bool {
        if self.name == ".." || self.name == "." {
            self.is_dir()
        } else {
            false
        }
    }
}

#[derive(Debug, Default, Clone, BinRead)]
pub struct JKRArchive {
    pub header: JKRArchiveHeader,
    pub dataheader: JKRArchiveDataHeader,
    #[br(calc(Vec::with_capacity(dataheader.dirnodecount as usize)))]
    pub foldernodes: Vec<JKRFolderNode>,
    #[br(calc(Vec::with_capacity(dataheader.filenodecount as usize)))]
    pub filenodes: Vec<JKRFileNode>
}

impl JKRArchive {
    pub fn read<R: BinReaderExt>(stream: &mut R) -> BinResult<Self> {
        let mut magic = [0u8; 4];
        stream.read_exact(&mut magic)?;
        let mgc = String::from(String::from_utf8_lossy(&magic));
        let endian = match mgc.as_str() {
            "RARC" => Endian::Big,
            "CRAR" => Endian::Little,
            _ => Endian::NATIVE
        };
        let mut archive: JKRArchive = stream.read_type(endian)?;
        let table = StringTable::from_archive(&archive, stream)?;
        let headersize = archive.header.headersize as u64;
        stream.seek(Start(archive.dataheader.dirnodeoffset as u64 + headersize))?;
        for i in 0..archive.dataheader.dirnodecount {
            let mut node: JKRFolderNode = stream.read_type(endian)?;
            node.name = table[node.node.nameoffs].clone();
            if i == 0 {
                node.isroot = true;
            }
            archive.foldernodes.push(node);
        }
        stream.seek(Start(archive.dataheader.filenodeoffset as u64 + headersize))?;
        for i in 0..archive.dataheader.filenodecount {
            let mut dir: JKRFileNode = stream.read_type(endian)?;
            dir.attr = JKRFileAttr::from(dir.node.attrandnameoffs >> 24);
            dir.nameoffs = (dir.node.attrandnameoffs & 0x00FFFFFF) as u16;
            stream.seek(Current(4))?;
            dir.name = table[dir.nameoffs as u32].clone();
            if dir.is_dir() && dir.node.data != u32::MAX {
                let index = dir.node.data as usize;
                dir.foldernode = Some(dir.node.data as usize);
                let fnode = &mut archive.foldernodes[index];
                if dir.node.hash == fnode.node.hash {
                    fnode.filenode = Some(i as usize);
                }
            } else if dir.is_file() {
                let pos = archive.header.filedataoffset as u64 + headersize + dir.node.data as u64;
                let size = dir.node.datasize as usize;
                dir.data = stream.seek_task(Start(pos), |x| {
                    let mut data = vec![0u8; size];
                    x.read(&mut data)?;
                    Ok(data)
                })?;
            }
            archive.filenodes.push(dir);
        }
        let mut i = 0;
        for node in &mut archive.foldernodes {
            let off = node.node.firstfileoff as usize;
            let count = node.node.filecount as usize;
            for j in off..(off+count) {
                archive.filenodes[j].parentnode = Some(i);
                node.childnodes.push(j);
            }
            i += 1;
        }
        Ok(archive)
    }

    fn get_children(&self, node: &JKRFolderNode) -> Vec<&JKRFileNode> {
        self.filenodes.iter().enumerate().filter(|(x, _)| node.childnodes.contains(x))
        .map(|(_, x)| x).collect()
    }
    fn get_folder(&self, node: &JKRFileNode) -> Option<&JKRFolderNode> {
        if let Some(index) = node.foldernode {
            Some(&self.foldernodes[index])
        } else {
            None
        }
    }
    fn get_parent(&self, node: &JKRFileNode) -> Option<&JKRFolderNode> {
        if let Some(index) = node.parentnode {
            Some(&self.foldernodes[index])
        } else {
            None
        }
    }
    fn get_file(&self, node: &JKRFolderNode) -> Option<&JKRFileNode> {
        if let Some(index) = node.filenode {
            Some(&self.filenodes[index])
        } else {
            None
        }
    }
    
    fn get_root_from_file(&self, node: &JKRFileNode) -> Vec<&JKRFolderNode> {
        let mut node = node.clone();
        let mut result = vec![];
        while let Some(folder) = self.get_parent(&node) {
            if !folder.isroot {
                result.push(folder);
                let dirs = self.get_children(folder);
                node = dirs[dirs.len() - 1].clone();
            } else {
                result.push(folder);
                break;
            }
        }
        result.reverse();
        result
    }
    fn get_root_from_folder(&self, node: &JKRFolderNode) -> Vec<&JKRFolderNode> {
        if let Some(file) = self.get_file(node) {
            self.get_root_from_file(file)
        }
        else {
            vec![]
        }
    }

    fn unpack_node<A: AsRef<Path>>(&self, node: &JKRFolderNode, dir: A) -> std::io::Result<()> {
        let dir = PathBuf::from(dir.as_ref());
        if node.isroot {
            std::fs::create_dir_all(dir.join(&node.name))?;
        }
        std::fs::create_dir_all(&dir)?;
        for child in self.get_children(node) {
            if child.name == "." || child.name == ".." {
                continue;
            }
            let fullpath = match dir.to_string_lossy().contains(&node.name) {
                true => dir.join(&child.name),
                false => dir.join(&node.name).join(&child.name)
            };
            if child.is_dir() {
                std::fs::create_dir_all(&fullpath)?;
                if let Some(folder) = self.get_folder(child) {
                    self.unpack_node(folder, fullpath)?;
                }
            } else if child.is_file() {
                std::fs::write(fullpath, &child.data)?;
            }
        }
        Ok(())
    }
    pub fn unpack<A: AsRef<Path>>(&self, dir: A) -> std::io::Result<()> {
        self.unpack_node(&self.foldernodes[0], dir)
    } 
}