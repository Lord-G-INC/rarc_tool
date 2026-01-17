pub mod directory;
pub mod file;

use super::Reference;

/// Hash calculation method used for Archive.
/// This is not the same hash calculation method BCSVs use.
/// Code inspired by pyjkernel.
pub const fn calc_hash(name: &str) -> u16 {
    let mut result = 0u16;
    let data = name.as_bytes();
    let mut i = 0;
    while i < data.len() {
        let ch = data[i] as u16;
        result = result.wrapping_mul(3).wrapping_add(ch);
        i += 1;
    }
    result
}

pub use directory::Directory;
pub use file::File;