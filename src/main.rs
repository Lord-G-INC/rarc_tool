#![allow(dead_code)]

mod rarc;
mod seektask;

use std::path::*;
use std::env;


fn main() {
    let args = env::args().collect::<Vec<String>>();
    let files = args.iter().skip(1).map(|x| Path::new(x))
    .filter(|x| x.exists() && x.is_file()).collect::<Vec<_>>();
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
}
