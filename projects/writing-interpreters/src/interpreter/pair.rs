use std::{cell::Cell, fmt};

use crate::interpreter::{taggedptr::Value, ScopedPtr};

use super::{
    error::SourcePos,
    printer::Print,
    safeptr::{MutatorScope, TaggedCellPtr, TaggedScopedPtr},
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

    pub fn set_first_source_code_pos(&self, pos: SourcePos) {
        self.first_pos.set(Some(pos));
    }

    pub fn set_second_source_code_pos(&self, pos: SourcePos) {
        self.second_pos.set(Some(pos));
    }

    pub fn dot<'guard>(&self, value: TaggedScopedPtr<'guard>) {
        self.second.set(value);
    }
}

impl Print for Pair {
    fn print<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        let mut tail = ScopedPtr::new(guard, self);

        write!(f, "({}", tail.first.get(guard))?;

        while let Value::Pair(next) = *tail.second.get(guard) {
            tail = next;
            write!(f, " {}", tail.first.get(guard))?;
        }

        // clunky way to print anything but nil
        let second = *tail.second.get(guard);
        match second {
            Value::Nil => (),
            _ => write!(f, " . {}", second)?,
        }

        write!(f, ")")
    }

    // In debug print, use dot notation
    fn debug<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            f,
            "({:?} . {:?})",
            self.first.get(guard),
            self.second.get(guard)
        )
    }
}
