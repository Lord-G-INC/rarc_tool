use std::io::{Seek, SeekFrom};
use binrw::BinResult;


pub trait SeekTask: Seek {
    fn seek_task<R, F: FnOnce(&mut Self) -> BinResult<R>>(&mut self, seekto: SeekFrom, func: F) -> BinResult<R> {
        let pos = self.stream_position()?;
        self.seek(seekto)?;
        let res = func(self);
        self.seek(SeekFrom::Start(pos))?;
        res
    }
}

impl<S: Seek> SeekTask for S {}