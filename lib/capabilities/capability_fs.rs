use super::{permission::Permission};

pub trait FS {
    fn fs(&self, path: &str) -> Self;
}

impl FS for Permission {
    fn fs(&self, path: &str) -> Self {
        todo!()
    }
}
