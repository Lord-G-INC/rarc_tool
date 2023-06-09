use yaz0::{Yaz0Archive, Yaz0Writer, Error as Yaz0Error, CompressionLevel};
use binrw::BinReaderExt;
use std::thread;
use std::sync::mpsc::channel;
use std::io::{Cursor, Error, ErrorKind};

pub fn decompress<R: BinReaderExt>(reader: &mut R) -> Result<Vec<u8>, Yaz0Error> {
    let mut arch = Yaz0Archive::new(reader)?;
    arch.decompress()
}

pub fn compress(data: &[u8], level: CompressionLevel) -> Result<Vec<u8>, Yaz0Error> {
    let scope = thread::scope(|scope| {
        let (send, revc) = channel();
        let task = scope.spawn::
        <_, Result<Vec<u8>, Yaz0Error>>(move || {
            let mut cursor = Cursor::new(vec![0u8; 0]);
            let writer = Yaz0Writer::new(&mut cursor);
            writer.compress_and_write_with_progress(data, level, send)?;
            Ok(cursor.into_inner())
        });
        while let Ok(msg) = revc.try_recv() {
            print!("{} out of {} written", msg.read_head, data.len());
            print!("\x1B[2J\x1B[1;1H");
        }
        println!("{} out of {} written", data.len(), data.len());
        task.join()
    });
    let err =
    Err(Yaz0Error::Io(Error::new(ErrorKind::Other, "Something went wrong")));
    scope.unwrap_or(err)
}