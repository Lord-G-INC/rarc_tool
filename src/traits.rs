use std::io::{Seek, SeekFrom, BufRead};
use std::io;

pub trait SeekTask: Seek {
    fn seektask<R, F: FnMut(&mut Self) -> io::Result<R>>(&mut self, pos: SeekFrom, func: F) 
    -> io::Result<R>;
}

impl <S: Seek> SeekTask for S {
    fn seektask<R, F: FnMut(&mut Self) -> io::Result<R>>(&mut self, pos: SeekFrom, mut func: F) 
    -> io::Result<R> {
        let cpos = self.seek(SeekFrom::Current(0))?;
        self.seek(pos)?;
        let res = func(self)?;
        self.seek(SeekFrom::Start(cpos))?;
        Ok(res)
    }
}

pub fn readntstringat<S: SeekTask + BufRead>(reader: &mut S, pos: SeekFrom) -> io::Result<String> {
    reader.seektask(pos, |task| {
        let mut buf = vec![];
        task.read_until(0, &mut buf)?;
        if buf.len() > 1 {
            buf.remove(buf.len() - 1);
        }
        Ok(String::from_utf8_lossy(&buf).into())
    })
}

pub trait LastPos {
    fn lastpos(&self) -> usize;
}

impl<T> LastPos for &[T] {
    fn lastpos(&self) -> usize {
        match self.len() {
            0 => 0,
            l => l - 1
        }
    }
}