use std::{io::{Cursor, SeekFrom}, path::Path};

use super::{Reference, header::{*, self}, nodes::*, make_reference};
use super::nodes::file::FileAttr;
use super::table::Table;
use binrw::prelude::*;

#[derive(Debug, Default, Clone)]
/// A JKRArchive. Contains everything needed to unpack, repack and modify the 
/// files and folders inside.
pub struct Archive {
    pub header: Header,
    pub data_header: DataHeader,
    pub folders: Vec<Reference<Directory>>,
    pub files: Vec<Reference<File>>,
    pub root: Reference<Directory>
}

impl Archive {
    pub const fn sync(&self) -> bool {
        self.data_header.sync
    }
    pub const fn sync_mut(&mut self) -> &mut bool {
        &mut self.data_header.sync
    }
    pub const fn next_id(&self) -> u16 {
        self.data_header.next_idx
    }
    pub const fn next_id_mut(&mut self) -> &mut u16 {
        &mut self.data_header.next_idx
    }

    pub fn read<R: BinReaderExt>(&mut self, reader: &mut R) -> BinResult<()> {
        let endian;
        (endian, self.header, self.data_header) = header::read_headers(reader)?;
        self.folders.reserve_exact(self.data_header.dir_node_count as _);
        self.files.reserve_exact(self.data_header.file_node_count as _);
        let table = self.read_table(reader)?;
        let mut off = self.data_header.dir_node_off + self.header.data_header_off;
        reader.seek(SeekFrom::Start(off as _))?;
        for i in 0..self.data_header.dir_node_count {
            let mut node = Directory::default();
            node.read(reader, endian)?;
            node.name = table[&node.node.name_off].clone();
            if i == 0 {
                node.is_root = true;
                self.root = make_reference(node);
                self.folders.push(self.root.clone());
            } else {
                self.folders.push(make_reference(node));
            }
        }
        off = self.data_header.file_node_off + self.header.data_header_off;
        reader.seek(SeekFrom::Start(off as _))?;
        for _ in 0..self.data_header.file_node_count {
            let mut node = File::default();
            node.read(reader, endian)?;
            node.name = table[&(node.name_off as u32)].clone();
            reader.seek(SeekFrom::Current(4))?;
            let node = make_reference(node);
            {
                let mut nlock = node.borrow_mut();
                if nlock.is_dir() && nlock.node.data != u32::MAX {
                    let index = nlock.node.data as usize;
                    let folder = &self.folders[index];
                    nlock.folder = Some(folder.clone());
                    let mut lock = folder.borrow_mut();
                    if lock.node.hash == nlock.node.hash {
                        lock.file = Some(node.clone());
                    }
                } else if nlock.is_file() {
                    let pos = self.header.file_data_off + self.header.data_header_off + nlock.node.data;
                    let len = nlock.node.data_size as _;
                    nlock.data.resize(len, 0);
                    let current = reader.stream_position()?;
                    reader.seek(SeekFrom::Start(pos as _))?;
                    reader.read_exact(&mut nlock.data)?;
                    reader.seek(SeekFrom::Start(current))?;
                }
            }
            self.files.push(node);
        }
        for node in &self.folders {
            let mut lock = node.borrow_mut();
            let off = lock.node.file_off as usize;
                let count = lock.node.file_count as usize;
                for i in off..(off + count) {
                    let file = &self.files[i];
                    let mut fileref = file.borrow_mut();
                    fileref.parent = Some(node.clone());
                    lock.children.push(file.clone());
                }
        }
        Ok(())
    }
    pub fn unpack<A: AsRef<Path>>(&self, dir: A) -> std::io::Result<()> {
        self.root.borrow().unpack(dir)
    }

    fn recalc_file_indicies(&mut self) {
        if self.sync() {
            *self.next_id_mut() = self.files.len() as u16;
            for i in 0..self.files.len() {
                let f = &self.files[i];
                let mut file = f.borrow_mut();
                if file.is_file() {
                    file.node.id = i as u16;
                }
            }
        } else {
            let mut id = 0u16;
            for f in &self.files {
                let mut file = f.borrow_mut();
                if file.is_file() {
                    file.node.id = id;
                    id += 1;
                }
            }
            *self.next_id_mut() = id;
        }
    }

    fn sort_nodes(&mut self, node: Reference<Directory>) {
        let mut shortcuts = Vec::new();
        let mut folders = Vec::new();
        for file in node.borrow().children.clone() {
            if file.borrow().is_shortcut() {
                shortcuts.push(file);
            } else if file.borrow().is_dir() {
                folders.push(file);
            }
        }
        for shortcut in shortcuts {
            let index = match shortcut.borrow().folder.as_ref() {
                Some(folder) => {
                    self.folders.iter()
                    .position(|x| {
                        x == folder
                    })
                    .map(|x| x as u32).unwrap()
                },
                None => u32::MAX
            };
            shortcut.borrow_mut().node.data = index;
            let shidx = node.borrow().children.iter()
            .position(|x| x == &shortcut).unwrap();
            node.borrow_mut().children.remove(shidx as usize);
            node.borrow_mut().children.push(shortcut.clone());
        }
        node.borrow_mut().node.file_off = self.files.len() as u32;
        let child_count = node.borrow().children.len() as u16;
        node.borrow_mut().node.file_count = child_count;
        for child in &node.borrow().children {
            self.files.push(child.clone());
        }
        for dir in folders {
            if let Ok(mut dir_ref) = dir.try_borrow_mut() {
                let folder = dir_ref.folder.as_ref().unwrap();
                let index = self.folders.iter()
                    .position(|x| {
                        x == folder
                    }).unwrap();
                dir_ref.node.data = index as u32;
            }
            let folder = {
                let dref = dir.borrow();
                let folder = dref.folder.as_ref().unwrap();
                folder.clone()
            };
            self.sort_nodes(folder);
        }
    }

    pub fn sort(&mut self) {
        self.files.clear();
        self.sort_nodes(self.root.clone());
        self.recalc_file_indicies();
    }

    pub fn create_folder<A: AsRef<str>>(&mut self, name: A, parent: Option<Reference<Directory>>)
        -> Reference<Directory> {
        let name = name.as_ref();
        let mut dir = Directory::default();
        dir.name = name.into();
        dir.node.short_name = dir.short_name();
        let true_dir = make_reference(dir);
        let file = 
        File::create(name, FileAttr::FOLDER,
            Some(true_dir.clone()), parent.clone());
        File::create(".", FileAttr::FOLDER,
            Some(true_dir.clone()), Some(true_dir.clone()));
        File::create("..", FileAttr::FOLDER,
        parent.clone(), Some(true_dir.clone()));
        true_dir.borrow_mut().file = Some(file);
        self.folders.push(true_dir.clone());
        true_dir
    }

    pub fn create_file<A: AsRef<str>>(&mut self, name: A, attr: FileAttr, parent: Option<Reference<Directory>>) 
        -> Reference<File> {
        let file = File::create(name.as_ref(), attr, None, parent.clone());
        if !self.sync() {
            file.borrow_mut().node.id = self.next_id();
            *self.next_id_mut() += 1;
        }
        file
    }

    fn create_root(&mut self, name: &str) {
        let mut item = Directory {
            name: name.into(),
            is_root: true,
            ..Default::default()
        };
        item.node.short_name = item.short_name();
        self.root = make_reference(item);
        self.folders.push(self.root.clone());
        File::create(".", FileAttr::FOLDER,
        Some(self.root.clone()), Some(self.root.clone()));
        File::create("..", FileAttr::FOLDER,
        None, Some(self.root.clone()));
    }

    pub fn create<A: AsRef<str>>(name: A, sync: bool) -> Self  {
        let mut arch = Self::default();
        *arch.sync_mut() = sync;
        arch.create_root(name.as_ref());
        arch
    }

    pub fn import<A: AsRef<Path>>(&mut self, path: A, attr: FileAttr) 
        -> std::io::Result<()> {
        self.import_node(path, attr, Some(self.root.clone()))?;
        self.sort();
        Ok(())
    }

    fn import_node<A: AsRef<Path>>(&mut self, path: A, attr: FileAttr, parent: Option<Reference<Directory>>) 
        -> std::io::Result<()> {
        let path = path.as_ref();
        if !path.is_dir() {
            return Ok(());
        }
        for entry in path.read_dir()? {
            let entry = entry?;
            let name = entry.file_name()
            .to_string_lossy().into_owned();
            if name == "." || name == ".." {
                continue;
            }
            if entry.path().is_dir() {
                let node = 
                self.create_folder(name, parent.clone());
                self.import_node(entry.path(), attr, Some(node))?;
            } else if entry.path().is_file() {
                let node = self.create_file(name, attr, parent.clone());
                node.borrow_mut().data = std::fs::read(entry.path())?;
                let size = node.borrow().data.len() as u32;
                node.borrow_mut().node.data_size = size;
            }
        }
        Ok(())
    }

    pub fn gen_table(&self) -> Table {
        let mut table = Table::default();
        table.add(".");
        table.add("..");
        table.add(&self.root.borrow().name);
        self.root.borrow_mut().node.name_off = 5;
        collect_strings(&mut table, self.root.clone());
        table
    }

    pub fn write<W: BinWriterExt>(&self, writer: &mut W, endian: binrw::Endian) -> BinResult<()> {
        let mut mram = vec![];
        let mut aram = vec![];
        let mut dvd = vec![];
        for file in &self.files {
            let fref = file.borrow();
            if fref.attr.contains(FileAttr::LOAD_TO_MRAM) {
                mram.push(file.clone());
            } else if fref.attr.contains(FileAttr::LOAD_TO_ARAM) {
                aram.push(file.clone());
            } else if fref.attr.contains(FileAttr::LOAD_FROM_DVD) {
                dvd.push(file.clone());
            }
        }
        let dnodecount = self.folders.len() as u32;
        let fnodecount = self.files.len() as u32;
        let dnodeoff = 0x40 + 
            u32::next_multiple_of(dnodecount * 0x10, 32);
        let stroff = dnodeoff +
            u32::next_multiple_of(fnodecount * 0x14, 32);
        let strdata = self.gen_table();
        writer.seek(SeekFrom::Start(stroff as u64))?;
        strdata.write(writer)?;
        writer.seek(SeekFrom::Start(0x40))?;
        for folder in &self.folders {
            folder.borrow().write(writer,endian)?;
        }
        let end = writer.seek(SeekFrom::End(0))?.next_multiple_of(32);
        while writer.stream_position()? < end {
            0u8.write_ne(writer)?;
        }
        let fdataoff = writer.stream_position()? as u32 - 0x20;
        let mram_size = write_file_data(writer, mram)?;
        let aram_size = write_file_data(writer, aram)?;
        let dvd_size = write_file_data(writer, dvd)?;
        let total_size = mram_size + aram_size + dvd_size;
        writer.seek(SeekFrom::Start(dnodeoff as u64))?;
        for file in &self.files {
            file.borrow().write(writer, endian)?;
            0i32.write_ne(writer)?;
        }
        let whole_size = writer.seek(SeekFrom::End(0))? as u32;
        writer.seek(SeekFrom::Start(0))?;
        Magic::from_endian(endian).write_ne(writer)?;
        let header = Header {
            size: whole_size,
            data_header_off: 0x20,
            file_data_off: fdataoff,
            file_data_len: total_size,
            mram_size,
            aram_size,
            dvd_size
        };
        header.write_options(writer, endian, ())?;
        let dataheader = DataHeader {
            dir_node_count: dnodecount,
            dir_node_off: 0x20,
            file_node_count: fnodecount,
            file_node_off: dnodeoff - 0x20,
            string_tbl_size: strdata.total_size(),
            string_tbl_off: stroff - 0x20,
            next_idx: self.next_id(),
            sync: self.sync()
        };
        dataheader.write_options(writer, endian, ())?;
        Ok(())
    }

    pub fn to_bytes(&self, endian: binrw::Endian) -> BinResult<Vec<u8>> {
        let mut writer = Cursor::new(vec![]);
        self.write(&mut writer, endian)?;
        Ok(writer.into_inner())
    }
}

fn collect_strings(table: &mut Table, node: Reference<Directory>) {
    let node = node.borrow();
    for i in 0..node.children.len() {
        let mut child = node.children[i].borrow_mut();
        if child.name == "." {
            child.name_off = 0;
        } else if child.name == ".." {
            child.name_off = 2;
        } else {
            child.name_off = table.add(&child.name) as u16;
        }
        if child.is_dir() && !child.is_shortcut() {
            if let Some(folder) = &child.folder {
                {
                    let mut folder = folder.borrow_mut();
                    folder.node.name_off = child.name_off as u32;
                }
                collect_strings(table, folder.clone());
            }
        }
    }
}

fn write_file_data<W: BinWriterExt>(writer: &mut W, files: Vec<Reference<File>>) -> BinResult<u32> {
    let start = writer.stream_position()?;
    for i in 0..files.len() {
        let mut file = files[i].borrow_mut();
        file.node.data = (writer.stream_position()? - start) as u32;
        writer.write_all(&file.data)?;
        while writer.stream_position()? % 32 != 0 {
            0u8.write_ne(writer)?;
        }
    }
    Ok((writer.stream_position()? - start) as u32)
}