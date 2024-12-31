use std::ptr::NonNull;

pub struct RawPtr<T: Sized> {
    ptr: NonNull<T>,
}
