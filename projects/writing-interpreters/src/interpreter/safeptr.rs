use std::cell::Cell;

use crate::memory::RawPtr;

use super::pointerops::ScopedRef;

/// Type that provides a generic anchor for mutator timeslice lifetimes
pub trait MutatorScope {}

/// An untagged compile-time typed pointer with scope limited by `MutatorScope`
pub struct ScopedPtr<'guard, T: Sized> {
    value: &'guard T,
}

impl<'guard, T: Sized> ScopedPtr<'guard, T> {
    pub fn new(_guard: &'guard dyn MutatorScope, value: &'guard T) -> ScopedPtr<'guard, T> {
        ScopedPtr { value }
    }
}

/// A wrapper around untagged raw pointers for storing compile-time typed pointers in data
/// structures with interior mutability, allowing pointers to be updated to point at different
/// target objects.
///
/// anywhere (structs, enums) that needs to store a pointer to something on the heap will use
/// CellPtr<T> and any code that accesses these pointers during the scope-guarded mutator code
/// will obtain ScopedPtr<T> instances that can be safely dereferenced.
#[derive(Clone)]
pub struct CellPtr<T: Sized> {
    inner: Cell<RawPtr<T>>,
}

impl<T: Sized> CellPtr<T> {
    pub fn get<'guard>(&self, guard: &'guard dyn MutatorScope) -> ScopedPtr<'guard, T> {
        ScopedPtr::new(guard, self.inner.get().scoped_ref(guard))
    }
}
