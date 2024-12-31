use crate::memory::{allocator::AllocObject, AllocRaw, RawPtr, StickyImmixHeap};

// GC and Rust: https://blog.pnkfx.org/blog/categories/gc/

use super::{
    error::RuntimeError,
    headers::{ObjectHeader, TypeList},
    pointerops::ScopedRef,
    safeptr::{MutatorScope, ScopedPtr},
    symbolmap::SymbolMap,
};

/// This type describes the mutator's view into memory - the heap and symbol name/ptr lookup.
///
/// It implements `MutatorScope` such that any `TaggedScopedPtr` or `Value` instances must be lifetime-
/// limited to the lifetime of this instance using `&'scope dyn MutatorScope`;
pub struct MutatorView<'memory> {
    heap: &'memory Heap,
}

impl<'memory> MutatorView<'memory> {
    fn new(mem: &'memory Memory) -> MutatorView<'memory> {
        MutatorView { heap: &mem.heap }
    }

    /// Write an object into the heap and return a scope-limited pointer to it    
    pub fn alloc<T>(&self, object: T) -> Result<ScopedPtr<'_, T>, RuntimeError>
    where
        T: AllocObject<TypeList>,
    {
        Ok(ScopedPtr::new(
            self,
            self.heap.alloc(object)?.scoped_ref(self),
        ))
    }
}

impl<'memory> MutatorScope for MutatorView<'memory> {}

pub type HeapStorage = StickyImmixHeap<ObjectHeader>;

/// Heap memory types.
struct Heap {
    heap: HeapStorage,
    syms: SymbolMap,
}

impl Heap {
    fn alloc<T>(&self, object: T) -> Result<RawPtr<T>, RuntimeError>
    where
        T: AllocObject<TypeList>,
    {
        Ok(self.heap.alloc(object)?)
    }
}

/// Wraps a heap and provides scope-limited access to the heap
pub struct Memory {
    heap: Heap,
}

impl Memory {
    pub fn mutate<M: Mutator>(&self, m: &M, input: M::Input) -> Result<M::Output, RuntimeError> {
        let mut guard = MutatorView::new(self);
        m.run(&mut guard, input)
    }
}

/// Defines the interface a heap-mutating type must use to be allowed access to the heap
/// If a piece of code wants to access the heap, it must implement this trait!
pub trait Mutator: Sized {
    type Input;
    type Output;

    fn run(&self, mem: &MutatorView, input: Self::Input) -> Result<Self::Output, RuntimeError>;
}
