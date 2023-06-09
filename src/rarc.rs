use crate::traits::*;
use binrw::prelude::*;
use binrw::Endian;
use bitflags::*;
use std::io::{Read, Seek, Write, BufRead, SeekFrom};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use serde_json;

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
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

impl BinRead for FileAttr {
    type Args<'a> = ();
    fn read_options<R: Read + Seek>(
            reader: &mut R,
            _: Endian,
            _: Self::Args<'_>,
        ) -> BinResult<Self> {
        Ok(Self::from_bits_retain(reader.read_ne()?))
    }
}

impl BinWrite for FileAttr {
    type Args<'a> = ();
    fn write_options<W: Write + Seek>(
            &self,
            writer: &mut W,
            _: Endian,
            _: Self::Args<'_>,
        ) -> BinResult<()> {
        writer.write_ne(&self.bits())
    }
}

impl Serialize for FileAttr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        self.bits().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FileAttr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        Ok(Self::from_bits_retain(u8::deserialize(deserializer)?))
    }
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PreloadType: i8 {
        const NONE = -1;
        const MRAM = 0;
        const ARAM = 1;
        const DVD = 2;
    }
}

impl BinRead for PreloadType {
    type Args<'a> = ();
    fn read_options<R: Read + Seek>(
            reader: &mut R,
            _: Endian,
            _: Self::Args<'_>,
        ) -> BinResult<Self> {
        Ok(Self::from_bits_retain(reader.read_ne()?))
    }
}

impl BinWrite for PreloadType {
    type Args<'a> = ();
    fn write_options<W: Write + Seek>(
            &self,
            writer: &mut W,
            _: Endian,
            _: Self::Args<'_>,
        ) -> BinResult<()> {
        writer.write_ne(&self.bits())
    }
}

#[derive(Debug, Clone, Copy, Default, BinRead, BinWrite)]
pub struct Header {
    pub filesize: u32,
    pub headersize: u32,
    pub filedataoff: u32,
    pub filedatasize: u32,
    pub mramsize: u32,
    pub aramsize: u32,
    pub dvdsize: u32
}

#[derive(Debug, Clone, Copy, Default, BinRead, BinWrite)]
pub struct DataHeader {
    pub dirnodecount: u32,
    pub dirnodeoff: u32,
    pub filenodecount: u32,
    pub filenodeoff: u32,
    pub stringtablesize: u32,
    pub stringtableoff: u32
}

#[derive(Debug, Clone, Copy, Default, BinRead, BinWrite, Serialize, Deserialize)]
pub struct FolderInfo {
    pub shortname: [u8; 4],
    pub nameoff: u32,
    pub hash: u16,
    pub filecount: u16,
    pub firstfileoff: u32
}

#[derive(Debug, Clone, Copy, Default, BinRead, BinWrite, Serialize, Deserialize)]
pub struct DirInfo {
    pub nodeidx: u16,
    pub hash: u16,
    pub attrandnameoff: u32,
    pub data: u32,
    pub datasize: u32
}

#[derive(Debug, Clone, Default, BinRead, BinWrite, Serialize, Deserialize)]
pub struct FileNode {
    pub info: FolderInfo,
    #[brw(ignore)]
    pub isroot: bool,
    #[brw(ignore)]
    pub name: String,
    #[serde(skip)]
    #[brw(ignore)]
    pub dir: Option<usize>
}

#[derive(Debug, Clone, Default, BinRead, BinWrite, Serialize, Deserialize)]
pub struct DirNode {
    pub info: DirInfo,
    #[brw(ignore)]
    pub attr: FileAttr,
    #[brw(ignore)]
    pub name: String,
    #[brw(ignore)]
    pub nameoff: u16,
    #[brw(ignore)]
    #[serde(skip)]
    pub data: Vec<u8>,
    #[brw(ignore)]
    #[serde(skip)]
    pub folder: Option<usize>,
    #[brw(ignore)]
    #[serde(skip)]
    pub parent: Option<usize>
}

#[derive(Debug, Clone, Default, BinRead)]
pub struct RARC {
    pub header: Header,
    pub dataheader: DataHeader,
    pub nextidx: u16,
    #[brw(ignore)]
    pub sync: bool,
    #[brw(ignore)]
    pub folders: Vec<FileNode>,
    #[brw(ignore)]
    pub dirs: Vec<DirNode>
}

impl RARC {
    pub fn read<R: BinReaderExt + BufRead>(reader: &mut R) -> BinResult<RARC> {
        let mut magic = vec![0u8; 4];
        reader.read_exact(&mut magic)?;
        let mgc = String::from(String::from_utf8_lossy(&magic));
        let endian = match mgc.as_str() {
            "CRAR" => Endian::Little,
            "RARC" => Endian::Big,
            _ => Endian::NATIVE,
        };
        let mut result: RARC = reader.read_type(endian)?;
        result.sync = reader.read_ne::<u8>()? != 0;
        let Self {header, dataheader, folders,
            dirs, ..} = &mut result;
        reader.seek(SeekFrom::Start((dataheader.dirnodeoff + header.headersize) as u64))?;
        folders.reserve_exact(dataheader.dirnodecount as usize);
        dirs.reserve_exact(dataheader.filenodecount as usize);
        for i in 0..folders.capacity() {
            let mut node = FileNode::read_options(reader, endian, ())?;
            let pos = (dataheader.stringtableoff + header.headersize + node.info.nameoff) as u64;
            node.name = readntstringat(reader, SeekFrom::Start(pos))?;
            node.isroot = i == 0;
            folders.push(node);
        }
        reader.seek(SeekFrom::Start((dataheader.filenodeoff + header.headersize) as u64))?;
        for _ in 0..dirs.capacity() {
            let mut node = DirNode::read_options(reader, endian, ())?;
            reader.seek(SeekFrom::Current(4))?;
            node.nameoff = (node.info.attrandnameoff & 0x00FFFFFF) as u16;
            node.attr = FileAttr::from_bits_retain((node.info.attrandnameoff >> 24) as u8);
            let pos = (dataheader.stringtableoff + header.headersize + node.nameoff as u32) as u64;
            node.name = readntstringat(reader, SeekFrom::Start(pos))?;
            if node.attr.contains(FileAttr::FILE) {
                let pos = (header.filedataoff + header.headersize + node.info.data) as u64;
                node.data = reader.seektask(SeekFrom::Start(pos), |task| {
                    let mut vec = vec![0u8; node.info.datasize as usize];
                    task.read_exact(&mut vec)?;
                    Ok(vec)
                })?;
            }
            dirs.push(node);
        }
        result.findparents();
        Ok(result)
    }
    fn findparents(&mut self) {
        for i in 0..self.dirs.len() {
            let dir = &mut self.dirs[i];
            if dir.attr.contains(FileAttr::FOLDER) && dir.info.data != u32::MAX {
                dir.folder = Some(dir.info.data as usize);
                if dir.info.hash == self.folders[dir.info.data as usize].info.hash {
                    self.folders[dir.info.data as usize].dir = Some(i);
                }
            }
        }
        for i in 0..self.folders.len() {
            let folder = &self.folders[i];
            for y in folder.info.firstfileoff..(folder.info.firstfileoff+folder.info.filecount as u32) {
                self.dirs[y as usize].folder = Some(i);
            }
        }
    }
    fn getchildren(&self, node: &FileNode) -> Vec<&DirNode> {
        let mut idxs = vec![];
        for y in node.info.firstfileoff..(node.info.firstfileoff+node.info.filecount as u32) {
            idxs.push(y as usize);
        }
        self.dirs.iter().enumerate().filter(|(x, _)| idxs.contains(x)).map(|(_, x)| x)
        .collect()
    }
    fn findfolder(&self, dir: &DirNode) -> Option<&FileNode> {
        match dir.folder {
            Some(n) => Some(&self.folders[n]),
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
    pub fn extract(&mut self) -> BinResult<()> {
        for folder in &self.folders {
            let children = self.getchildren(folder);
            let tree = self.getroot(&children);
            let mut path = PathBuf::from(tree[0].name.clone());
            for t in 1..tree.len() {
                path.push(&tree[t].name);
            }
            std::fs::create_dir_all(&path)?;
            for child in children.iter()
            .filter(|x| x.attr.contains(FileAttr::FILE)) {
                std::fs::write(path.join(&child.name), &child.data)?;
            }
            let fptr = children[children.len() - 2];
            let rptr = children[children.len() - 1];
            let mut msg = serde_json::to_string_pretty(fptr).unwrap_or_default();
            std::fs::write("folder.json", &msg)?;
            msg = serde_json::to_string_pretty(rptr).unwrap_or_default();
            std::fs::write("parent.json", msg)?;
        }
        Ok(())
    }
}