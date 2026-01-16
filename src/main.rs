use std::io::Cursor;
use rarc_lib::*;

fn main() -> binrw::BinResult<()> {
    let mut data = std::fs::read("AbekobeGalaxyMap.arc")?;
    data = decompres_yaz0(data);
    let mut reader = Cursor::new(data);
    let mut archive = Archive::default();
    archive.read(&mut reader)?;
    archive.unpack(std::env::current_dir()?)?;
    archive = Archive::create("AbekobeGalaxyMap", true);
    archive.import("Stage", FileAttr::FILE | FileAttr::LOAD_TO_MRAM)?;
    let mut save = archive.to_bytes(binrw::Endian::Big)?;
    save = compress_yaz0(save, yaz0::CompressionLevel::Lookahead { quality: 7 });
    std::fs::write("test.arc", save)?;
    Ok(())
}
