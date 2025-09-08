#![allow(dead_code)]

use std::{error::Error, io::Cursor};

mod util;
mod types;
mod archive;
mod enums;
mod hash;
mod string_table;
mod seek_task;

fn main() -> Result<(), Box<dyn Error>> {
    let path = "WarpAreaErrorLayout.arc";
    let mut data = std::fs::read(path)?;
    let magic = String::from_utf8_lossy(&data[0..4]);
    data = match magic.eq("Yaz0") {
        true => util::decode(data)?,
        false => data
    };
    let mut cursor = Cursor::new(data);
    let mut arch = archive::JKRArchive::default();
    arch.read(&mut cursor)?;
    arch.unpack(std::env::current_dir()?)?;
    Ok(())
}
