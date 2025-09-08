pub const fn calchash(str: &str) -> u16 {
    let mut ret = 0u16;
    let bytes = str.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i] as u16;
        ret = ret.wrapping_mul(3);
        ret = ret.wrapping_add(b);
        i += 1;
    }
    ret
}