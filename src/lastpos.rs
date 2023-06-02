pub trait LastPos {
    fn lastpos(&self) -> usize;
}

impl<T> LastPos for Vec<T> {
    fn lastpos(&self) -> usize {
        self.len() - 1
    }
}