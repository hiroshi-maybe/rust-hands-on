use crate::memory::ArraySize;

use super::{safeptr::MutatorScope, MutatorView, RuntimeError};

/// Base container-type trait. All container types are subtypes of `Container`.
///
/// All container operations _must_ follow interior mutability only rules.
/// Because there are no compile-time mutable aliasing guarantees, there can be no references
/// into arrays at all, unless there can be a guarantee that the array memory will not be
/// reallocated.
///
/// `T` cannot be restricted to `Copy` because of the use of `Cell` for interior mutability.
pub trait Container<T: Sized + Clone>: Sized {
    /// Create a new, empty container instance.
    fn new() -> Self;
    /// Create a new container instance with the given capacity.
    // TODO: this may not make sense for tree types
    fn with_capacity<'guard>(
        mem: &'guard MutatorView,
        capacity: ArraySize,
    ) -> Result<Self, RuntimeError>;

    /// Reset the size of the container to zero - empty
    fn clear<'guard>(&self, mem: &'guard MutatorView) -> Result<(), RuntimeError>;

    /// Count of items in the container
    fn length(&self) -> ArraySize;
}

/// Generic stack trait. If implemented, the container can function as a stack
pub trait StackContainer<T: Sized + Clone>: Container<T> {
    /// Push can trigger an underlying array resize, hence it requires the ability to allocate
    fn push<'guard>(&self, mem: &'guard MutatorView, item: T) -> Result<(), RuntimeError>;

    /// Pop returns a bounds error if the container is empty, otherwise moves the last item of the
    /// array out to the caller.
    fn pop<'guard>(&self, _guard: &'guard dyn MutatorScope) -> Result<T, RuntimeError>;

    /// Return the value at the top of the stack without removing it
    fn top<'guard>(&self, _guard: &'guard dyn MutatorScope) -> Result<T, RuntimeError>;
}
