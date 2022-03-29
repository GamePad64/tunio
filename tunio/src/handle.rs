use core::marker::{Send, Sync};

pub struct UnsafeHandle<T>(pub T);

unsafe impl<T> Send for UnsafeHandle<T> {}

unsafe impl<T> Sync for UnsafeHandle<T> {}
