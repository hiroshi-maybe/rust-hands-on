use std::cell::Cell;

use crate::memory::ArraySize;

use super::rawarray::RawArray;

/// An array, like Vec, but applying an interior mutability pattern.
///
/// Implements Container traits, including SliceableContainer.
/// Since SliceableContainer allows mutable access to the interior
/// of the array, RefCell-style runtime semantics are employed to
/// prevent the array being modified outside of the slice borrow.
#[derive(Clone)]
pub struct Array<T: Sized + Clone> {
    length: Cell<ArraySize>,
    data: Cell<RawArray<T>>,
    // borrow: Cell<BorrowFlag>,
}

/// Array of u8
pub type ArrayU8 = Array<u8>;
/// Array of u16
pub type ArrayU16 = Array<u16>;
/// Array of u32
pub type ArrayU32 = Array<u32>;
