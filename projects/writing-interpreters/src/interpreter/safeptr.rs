use std::{cell::Cell, fmt, ops::Deref};

use crate::memory::RawPtr;

use super::{
    pointerops::ScopedRef,
    printer::Print,
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

impl<'guard, T: Sized> Deref for ScopedPtr<'guard, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value
    }
}

impl<'guard, T: Sized + Print> fmt::Display for ScopedPtr<'guard, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.print(self, f)
    }
}

impl<'guard, T: Sized + Print> fmt::Debug for ScopedPtr<'guard, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.print(self, f)
    }
}

/// Anything that _has_ a scope lifetime can pass as a scope representation
impl<'scope, T: Sized> MutatorScope for ScopedPtr<'scope, T> {}

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

    // the explicit 'guard lifetime bound to MutatorScope is omitted here since the ScopedPtr
    // carries this lifetime already so we can assume that this operation is safe
    pub fn set(&self, source: ScopedPtr<T>) {
        self.inner.set(RawPtr::new(source.value))
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

impl<'guard> Deref for TaggedScopedPtr<'guard> {
    type Target = Value<'guard>;

    fn deref(&self) -> &Value<'guard> {
        &self.value
    }
}

impl<'guard> fmt::Display for TaggedScopedPtr<'guard> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<'guard> fmt::Debug for TaggedScopedPtr<'guard> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
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

    /// Construct a new TaggedCellPtr from a TaggedScopedPtr
    pub fn new_with(source: TaggedScopedPtr) -> TaggedCellPtr {
        TaggedCellPtr {
            inner: Cell::new(TaggedPtr::from(source.ptr)),
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

    /// Take the pointer of another `TaggedCellPtr` and set this instance to point at that object too
    pub fn copy_from(&self, other: &TaggedCellPtr) {
        self.inner.set(other.inner.get());
    }
}
