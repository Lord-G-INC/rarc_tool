use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use binrw::prelude::*;


#[derive(Debug, Default, Clone)]
pub struct StringTable {
    pub table: HashMap<u32, String>,
    pub(crate) data: Vec<u8>
}


impl StringTable {
    pub fn from_data(data: &[u8]) -> Self {
        let mut result = Self::default();
        result.data = data.to_vec();
        let mut i = 0;
        let mut off = 0u32;
        while i < result.data.len() {
            let mut bytes = vec![0u8; 0];
            let mut byte = result.data[i];
            i += 1;
            while byte != 0 {
                bytes.push(byte);
                byte = result.data[i];
                i += 1;
            }
            let str = String::from(String::from_utf8_lossy(&bytes));
            println!("{}", &str);
            result.table.insert(off, str);
            off += bytes.len() as u32 + 1;
            if result.data[i..].iter().all(|x| *x == 0) {
                result.data.drain(i..);
                break;
            }
        }
        result
    }    

    pub fn reverse_table(&self) -> HashMap<String, u32> {
        self.table.iter().map(|(k, v)| (v.clone(), *k)).collect()
    }

    pub fn add<A: AsRef<str>>(&mut self, item: A) -> u32 {
        let key = String::from(item.as_ref());
        let reverse_table = self.reverse_table();
        if let Some(off) = reverse_table.get(&key) {
            *off
        } else {
            let o = self.data.len() as u32;
            let mut bytes = Vec::from(key.as_bytes());
            self.data.append(&mut bytes);
            self.data.push(0);
            self.table.insert(o, key);
            o
        }
    }
}

impl Index<u32> for StringTable {
    type Output = String;
    fn index(&self, index: u32) -> &Self::Output {
        &self.table[&index]
    }
}

impl IndexMut<u32> for StringTable {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.table.get_mut(&index).unwrap()
    }
}