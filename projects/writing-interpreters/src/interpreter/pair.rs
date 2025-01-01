use std::cell::Cell;

use super::{
    error::SourcePos,
    safeptr::{TaggedCellPtr, TaggedScopedPtr},
    MutatorView, RuntimeError,
};

/// A Pair of pointers, like a Cons cell of old
#[derive(Clone)]
pub struct Pair {
    pub first: TaggedCellPtr,
    pub second: TaggedCellPtr,
    // Possible source code positions of the first and second values
    pub first_pos: Cell<Option<SourcePos>>,
    pub second_pos: Cell<Option<SourcePos>>,
}

impl Pair {
    pub fn new() -> Pair {
        Pair {
            first: TaggedCellPtr::new_nil(),
            second: TaggedCellPtr::new_nil(),
            first_pos: Cell::new(None),
            second_pos: Cell::new(None),
        }
    }

    pub fn cons<'guard>(
        mem: &'guard MutatorView,
        head: TaggedScopedPtr<'guard>,
        rest: TaggedScopedPtr<'guard>,
    ) -> Result<TaggedScopedPtr<'guard>, RuntimeError> {
        let pair = Pair::new();
        pair.first.set(head);
        pair.second.set(rest);
        mem.alloc_tagged(pair)
    }

    pub fn append<'guard>(
        &self,
        mem: &'guard MutatorView,
        value: TaggedScopedPtr<'guard>,
    ) -> Result<TaggedScopedPtr<'guard>, RuntimeError> {
        let pair = Pair::new();
        pair.first.set(value);

        let pair = mem.alloc_tagged(pair)?;
        self.second.set(pair);

        Ok(pair)
    }

    pub fn dot<'guard>(&self, value: TaggedScopedPtr<'guard>) {
        self.second.set(value);
    }
}
