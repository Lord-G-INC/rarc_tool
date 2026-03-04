use crate::{Archive, Reference, nodes::*};

pub struct DirIter<'a> {
    archive: &'a Archive,
    name: String
}

impl<'a> DirIter<'a> {
    pub fn new<A: AsRef<str>>(archive: &'a Archive, name: A) -> Self {
        Self {archive, name: name.as_ref().into()}
    }
    pub fn find_matches(&self) -> Vec<Reference<Directory>> {
        let mut result = vec![];
        let DirIter { archive, name } = self;
        let root = &archive.root;
        Self::match_name(&mut result, root, name);
        result
    }
    fn match_name(vec: &mut Vec<Reference<Directory>>, 
        node: &Reference<Directory>, name: &String) {
        let borrow = node.borrow();
        if &borrow.name == name {
            vec.push(node.clone());
        }
        for child in &borrow.children {
            let cborrow = child.borrow();
            if let Some(dir) = &cborrow.folder {
                Self::match_name(vec, dir, name);
            }
        }
    }
}

pub struct FileIter<'a> {
    archive: &'a Archive,
    name: String
}

impl<'a> FileIter<'a> {
    pub fn new<A: AsRef<str>>(archive: &'a Archive, name: A) -> Self {
        Self {archive, name: name.as_ref().into()}
    }
    pub fn find_matches(&self) -> Vec<Reference<File>> {
        let mut result = vec![];
        let FileIter { archive, name } = self;
        Self::match_name(&mut result, &archive.root, name);
        result
    }
    fn match_name(vec: &mut Vec<Reference<File>>, node: &Reference<Directory>,
        name: &String) {
        let borrow = node.borrow();
        for child in &borrow.children {
            let cborrow = child.borrow();
            if cborrow.is_file() {
                if &cborrow.name == name {
                    vec.push(child.clone());
                }
            } else if let Some(folder) 
                = &cborrow.folder {
                Self::match_name(vec, folder, name);
            }
        }
    }
}


impl Archive {
    pub fn find_dirs_by_name<A: AsRef<str>>(&self, name: A) -> Vec<Reference<Directory>> {
        let iter = DirIter::new(self, name);
        iter.find_matches()
    }
    pub fn find_files_by_name<A: AsRef<str>>(&self, name: A) -> Vec<Reference<File>> {
        let iter = FileIter::new(self, name);
        iter.find_matches()
    }
}