use std::{cell::Cell, fmt};

use crate::memory::ArraySize;

use super::{printer::Print, rawarray::RawArray, safeptr::MutatorScope};

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

impl Print for ArrayU8 {
    fn print<'guard>(
        &self,
        _guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "ArrayU8[...]")
    }
}

impl Print for ArrayU16 {
    fn print<'guard>(
        &self,
        _guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "ArrayU16[...]")
    }
}

impl Print for ArrayU32 {
    fn print<'guard>(
        &self,
        _guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "ArrayU32[...]")
    }
}
