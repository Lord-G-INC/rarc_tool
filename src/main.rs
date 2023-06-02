#![allow(dead_code)]

mod rarc;
mod seektask;
mod lastpos;

use std::path::*;
use std::env;


fn main() {
    let args = env::args().collect::<Vec<String>>();
    let files = args.iter().skip(1).map(|x| Path::new(x))
    .filter(|x| x.exists() && x.is_file()).collect::<Vec<_>>();
    let dirs = args.iter().skip(1).map(|x| Path::new(x))
    .filter(|x| x.exists() && x.is_dir()).collect::<Vec<_>>();
    for file in files {
        let mut data = std::fs::read(file).unwrap_or_default();
        data = yaz0rust::decompress(&data);
        let rarc = rarc::RARC::read(data);
        if let Some(parent) = file.parent() {
            if parent.exists() {
                std::env::set_current_dir(parent).unwrap();
            }
        }
        rarc.extract();
    }
    for dir in dirs {
        let mut rarc = rarc::RARC::default();
        let attr = rarc::FileAttr::FILE | rarc::FileAttr::LOADTOMRAM;
        let filepath: String = dir.file_name().unwrap_or_default().to_string_lossy().into();
        rarc.importfromfolder(filepath, attr);
        let mut num = 0usize;
        for item in rarc.dirs {
            if item.name == ".." || item.name == "." {
                continue;
            }
            num += 1;
        }
        println!("{}", num);
    }
}
