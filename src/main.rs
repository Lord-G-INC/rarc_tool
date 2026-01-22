use std::{io::Cursor, path::PathBuf};
use rarc_lib::*;
use clap::*;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Endian {
    Big,
    Little
}

impl Default for Endian {
    fn default() -> Self {
        if cfg!(target_endian = "big") {
            Self::Big
        } else {
            Self::Little
        }
    }
}

impl From<Endian> for binrw::Endian {
    fn from(value: Endian) -> Self {
        match value {
            Endian::Big => binrw::Endian::Big,
            Endian::Little => binrw::Endian::Little
        }
    }
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum Attr {
    #[default]
    None,
    Mram,
    Aram,
    Dvd
}

impl From<Attr> for nodes::file::FileAttr {
    fn from(value: Attr) -> Self {
        match value {
            Attr::None => Self::FILE,
            Attr::Mram => Self::FILE | Self::LOAD_TO_MRAM,
            Attr::Aram => Self::FILE | Self::LOAD_TO_ARAM,
            Attr::Dvd => Self::FILE | Self::LOAD_FROM_DVD
        }
    }
}

#[derive(Parser, Clone, Debug, Default)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    /// The input, if this is a file, attempt to unpack the Archive.
    /// If this is a directory, attempt to make an Archive.
    pub input: PathBuf,
    #[arg(short, long)]
    /// Optional output, must be a dir when unpacking and a file when packing.
    pub output: Option<PathBuf>,
    #[arg(short, long, default_value = "big")]
    /// ByteOrder to use, defaults to big given the format is more used
    /// on the Wii which is big.
    pub endian: Endian,
    #[arg(short, long, default_value = "mram")]
    /// File attribute to use.
    /// 
    /// none does nothing.
    /// 
    /// mram loads to "main ram" (wii).
    /// 
    /// aram loads to "auxiliary ram" (gcn).
    /// 
    /// dvd loads right off the DVD when needed (wii, gcn).
    pub attr: Attr
}

fn main() -> binrw::BinResult<()> {
    let args = Args::parse();
    let Args { input, output, endian, attr } = args;
    if input.is_file() {
        let mut data = std::fs::read(&input)?;
        data = decompres_yaz0(data);
        let mut reader = Cursor::new(data);
        let mut archive = Archive::default();
        archive.read(&mut reader)?;
        let name = &archive.root.borrow().name;
        if let Some(parent) = input.parent()
            && !parent.to_string_lossy().is_empty() {
            archive.unpack(&parent)?;
            println!("Unpacked to {:?}", parent.join(name));
        } else {
            let dir = std::env::current_dir()?;
            archive.unpack(&dir)?;
            println!("Unpacked to {:?}", dir.join(name));
        }
    } else if input.is_dir() {
        let name = input.file_name().unwrap().to_string_lossy();
        let mut archive = Archive::create(name, true);
        archive.import(&input, attr.into())?;
        let mut data = archive.to_bytes(endian.into())?;
        let level = yaz0::CompressionLevel::Lookahead { quality: 7 };
        data = compress_yaz0(data, level);
        let mut size = data.len();
        size = size.next_multiple_of(32) - size;
        let mut extra = vec![0u8; size];
        data.append(&mut extra);
        if let Some(out) = &output {
            std::fs::write(out, data)?;
            println!("Packed to {:?}", std::path::absolute(out)?);
        } else {
            let path = input.with_extension("arc");
            std::fs::write(&path, data)?;
            println!("Packed to {:?}", std::path::absolute(path)?);
        }
    }
    Ok(())
}
