pub mod file;
pub mod folder;

use std::rc::Rc;
use std::cell::RefCell;

pub type RcCell<T> = Rc<RefCell<T>>;

pub fn move_shared<T>(item: T) -> RcCell<T> {
    RcCell::new(RefCell::new(item))
}

pub fn make_shared<T: Default>() -> RefCell<T> {
    Default::default()
}

pub use file::FileNode;
pub use folder::FolderNode;