use std::{rc::Rc, cell::RefCell};
use std::io::Cursor;

use yaz0::*;

pub mod header;
pub mod nodes;
pub mod archive;
pub mod table;
pub use binrw;
pub use yaz0;

/// Typedef for the Node types to make use of.
pub type Reference<T> = Rc<RefCell<T>>;

pub use archive::Archive;
pub use nodes::file::FileAttr;

/// Utility method to easily make a [Reference].
pub fn make_reference<T>(item: T) -> Reference<T> {
    Rc::new(RefCell::new(item))
}

pub fn decompres_yaz0<A: AsRef<[u8]>>(buf: A) -> Vec<u8> {
    let buf = Vec::from(buf.as_ref());
    let reader = Cursor::new(buf.clone());
    if let Ok(mut archive) = 
        Yaz0Archive::new(reader) &&
        let Ok(buffer) = archive.decompress() {
            buffer
        }
    else {
        buf
    }
}

pub fn compress_yaz0<A: AsRef<[u8]>>(buf: A, level: CompressionLevel) -> Vec<u8> {
    let mut writer = Cursor::new(vec![]);
    let yaz0 = Yaz0Writer::new(&mut writer);
    if let Ok(_) = yaz0.compress_and_write(buf.as_ref(), level) {
        writer.into_inner()
    } else {
        Vec::from(buf.as_ref())
    }
}