use std::io::{Seek, SeekFrom::{Current, Start}, BufRead};

pub struct SeekTask<'a, T: Seek> {
    pub reader: &'a mut T,
    pos: u64
}

impl <'a, T: Seek> SeekTask<'a, T> {
    pub fn new(reader: &'a mut T, npos: u64) -> Self {
        let pos = reader.seek(Current(0)).unwrap_or_default();
        let result = Self { reader, pos };
        result.reader.seek(Start(npos)).unwrap();
        result
    }
}

impl <'a, T: Seek> Drop for SeekTask<'a, T> {
    fn drop(&mut self) {
        self.reader.seek(Start(self.pos)).unwrap();
    }
}

pub fn readntstringat<'a, T: Seek + BufRead>(reader: &'a mut T, pos: u64) -> String {
    seektask(reader, pos, |task| {
        let mut buf = vec![];
        task.reader.read_until(0, &mut buf).unwrap();
        if buf.len() > 1 {
            buf.remove(buf.len() - 1);
        }
        String::from(String::from_utf8_lossy(&buf))
    })
}

pub fn seektask<'a, R, T: Seek, G: FnMut(SeekTask<'a,T>) -> R>
(reader: &'a mut T, pos: u64, mut func: G) -> R {
    let task = SeekTask::new(reader, pos);
    func(task)
}