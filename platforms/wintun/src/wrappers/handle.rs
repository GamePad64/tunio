use core::marker::{Send, Sync};

pub struct HandleWrapper<T: Copy>(pub T);

impl<T: Copy> HandleWrapper<T> {
    pub(crate) fn clone(&self) -> Self {
        Self(self.0)
    }
}

unsafe impl<T: Copy> Send for HandleWrapper<T> {}

unsafe impl<T: Copy> Sync for HandleWrapper<T> {}
