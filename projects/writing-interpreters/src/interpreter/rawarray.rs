use std::ptr::NonNull;

use crate::memory::ArraySize;

/// Fundamental array type on which other variable-length types are built.
/// Analagous to RawVec.
pub struct RawArray<T: Sized> {
    /// Count of T-sized objects that can fit in the array
    capacity: ArraySize,
    ptr: Option<NonNull<T>>,
}

/// Since this base array type needs to be used in an interior-mutable way by the containers
/// built on top of it, the Copy+Clone traits need to be implemented for it so that it can
/// be used in a Cell
impl<T: Sized> Clone for RawArray<T> {
    fn clone(&self) -> Self {
        RawArray {
            capacity: self.capacity,
            ptr: self.ptr,
        }
    }
}

impl<T: Sized> Copy for RawArray<T> {}
