use std::cell::Cell;

use crate::memory::RawPtr;

use super::{
    pointerops::ScopedRef,
    taggedptr::{FatPtr, TaggedPtr, Value},
};

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
impl<'guard, T: Sized> Clone for ScopedPtr<'guard, T> {
    fn clone(&self) -> ScopedPtr<'guard, T> {
        ScopedPtr { value: self.value }
    }
}
impl<'guard, T: Sized> Copy for ScopedPtr<'guard, T> {}

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
    /// Construct a new CellPtr from a ScopedPtr
    pub fn new_with(source: ScopedPtr<T>) -> CellPtr<T> {
        CellPtr {
            inner: Cell::new(RawPtr::new(source.value)),
        }
    }

    pub fn get<'guard>(&self, guard: &'guard dyn MutatorScope) -> ScopedPtr<'guard, T> {
        ScopedPtr::new(guard, self.inner.get().scoped_ref(guard))
    }
}

/// A _tagged_ runtime typed pointer type with scope limited by `MutatorScope` such that a `Value`
/// instance can safely be derived and accessed. This type is neccessary to derive `Value`s from.
#[derive(Copy, Clone)]
pub struct TaggedScopedPtr<'guard> {
    ptr: TaggedPtr,
    value: Value<'guard>,
}

impl<'guard> TaggedScopedPtr<'guard> {
    pub fn new(guard: &'guard dyn MutatorScope, ptr: TaggedPtr) -> TaggedScopedPtr<'guard> {
        TaggedScopedPtr {
            ptr,
            value: FatPtr::from(ptr).as_value(guard),
        }
    }
}

/// A wrapper around the runtime typed `TaggedPtr` for storing pointers in data structures with
/// interior mutability, allowing pointers to be updated to point at different target objects.
#[derive(Clone)]
pub struct TaggedCellPtr {
    inner: Cell<TaggedPtr>,
}

impl TaggedCellPtr {
    /// Construct a new Nil TaggedCellPtr instance
    pub fn new_nil() -> TaggedCellPtr {
        TaggedCellPtr {
            inner: Cell::new(TaggedPtr::nil()),
        }
    }

    /// Return the pointer as a `TaggedScopedPtr` type that carries a copy of the `TaggedPtr` and
    /// a `Value` type for both copying and access convenience
    pub fn get<'guard>(&self, guard: &'guard dyn MutatorScope) -> TaggedScopedPtr<'guard> {
        TaggedScopedPtr::new(guard, self.inner.get())
    }

    /// Set this pointer to point at the same object as a given `TaggedScopedPtr` instance
    /// The explicit 'guard lifetime bound to MutatorScope is omitted here since the TaggedScopedPtr
    /// carries this lifetime already so we can assume that this operation is safe
    pub fn set(&self, source: TaggedScopedPtr) {
        self.inner.set(TaggedPtr::from(source.ptr))
    }
}
