use std::ptr::NonNull;

pub struct RawPtr<T: Sized> {
    ptr: NonNull<T>,
}

impl<T: Sized> RawPtr<T> {
    pub fn new(ptr: *const T) -> RawPtr<T> {
        RawPtr {
            ptr: unsafe { NonNull::new_unchecked(ptr as *mut T) },
        }
    }

    pub fn as_ptr(self) -> *const T {
        self.ptr.as_ptr()
    }
}

impl<T: Sized> Clone for RawPtr<T> {
    fn clone(&self) -> Self {
        RawPtr { ptr: self.ptr }
    }
}

impl<T: Sized> Copy for RawPtr<T> {}
