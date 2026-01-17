use std::ffi::*;
use super::*;

#[unsafe(no_mangle)]
pub extern "C" fn archive_to_dir(buffer: *const u8, size: usize, dir: *const i8) -> bool {
    if buffer.is_null() || dir.is_null() {
        return false;
    }
    let buffer = unsafe { 
        std::slice::from_raw_parts(buffer, size) 
    };
    let dir = unsafe {
        CStr::from_ptr(dir)
    }.to_string_lossy().into_owned();
    let data = decompres_yaz0(buffer);
    let mut reader = Cursor::new(data);
    let mut archive = Archive::default();
    if let Ok(_) = archive.read(&mut reader) {
        let result = archive.unpack(&dir);
        result.is_ok()
    } else {
        false
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dir_to_archive(dir: *const i8, attr: u8, file: *const i8) -> bool {
    if dir.is_null() || file.is_null() {
        return false;
    }
    let directory = unsafe {
        CStr::from_ptr(dir)
    }.to_string_lossy().into_owned();
    let file_out = unsafe {
        CStr::from_ptr(file)
    }.to_string_lossy().into_owned();
    let mut archive = Archive::create(&directory, true);
    if let Ok(_) = archive.import(directory, FileAttr(attr)) {
        if let Ok(mut data) = archive.to_bytes(binrw::Endian::Big) {
            let level = yaz0::CompressionLevel::Lookahead { quality: 7 };
            data = compress_yaz0(data, level);
            std::fs::write(file_out, data).is_ok()
        } else {
            false
        }
    } else {
        false
    }
}