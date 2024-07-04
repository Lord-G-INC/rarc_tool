use std::num::Wrapping;

pub fn calchash(text: &str) -> u32 {
    let mut output = Wrapping(0u32);
    for char in text.bytes() {
        output = Wrapping(char as u32) + (output * Wrapping(0x1f));
    }
    output.0
}