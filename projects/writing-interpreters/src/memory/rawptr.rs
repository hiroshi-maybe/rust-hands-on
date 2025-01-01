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

    /// Get the pointer value as a word-sized integer
    pub fn as_word(self) -> usize {
        self.ptr.as_ptr() as usize
    }

    pub fn as_untyped(self) -> NonNull<()> {
        self.ptr.cast()
    }
}

impl<T: Sized> Clone for RawPtr<T> {
    fn clone(&self) -> Self {
        RawPtr { ptr: self.ptr }
    }
}

impl<T: Sized> Copy for RawPtr<T> {}
