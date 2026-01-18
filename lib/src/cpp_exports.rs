use cxx;
use super::*;

#[cxx::bridge(namespace = "librarc")]
mod ffi {
    extern "Rust" {
        fn archive_to_dir(data: &CxxVector<u8>, output: &CxxString) -> String;
        fn dir_to_archive(path: &CxxString, attr: u8, endian: u8) -> UniquePtr<CxxVector<u8>>;
    }
}

fn archive_to_dir(data: &cxx::CxxVector<u8>, output: &cxx::CxxString) -> String {
    let mut reader = Cursor::new(decompres_yaz0(data.as_slice()));
    let mut archive = Archive::default();
    if let Ok(_) = archive.read(&mut reader) {
        let dir = output.to_string_lossy().into_owned();
        if let Ok(result) = archive.unpack(dir) {
            return result.to_string_lossy().into_owned()
        }
    }
    Default::default()
}

fn dir_to_archive(path: &cxx::CxxString, attr: u8, endian: u8) -> cxx::UniquePtr<cxx::CxxVector<u8>> {
    let mut result = cxx::CxxVector::new();
    let path = path.to_string_lossy().into_owned();
    let mut archive = Archive::create(&path, true);
    if let Ok(_) = archive.import(&path, FileAttr(attr)) {
        let endian = match endian {
            0 => binrw::Endian::Big,
            1 => binrw::Endian::Little,
            _ => binrw::Endian::NATIVE
        };
        if let Ok(mut data) = archive.to_bytes(endian) {
            let level = CompressionLevel::Lookahead { quality: 7 };
            data = compress_yaz0(data, level);
            let mut pin = result.pin_mut();
            for byte in data {
                pin.as_mut().push(byte);
            }
        }
    }
    result
}