use std::{cell::Cell, fmt};

use crate::interpreter::{taggedptr::Value, ScopedPtr};

use super::{
    error::{err_eval, SourcePos},
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

/// Unpack a list of Pair instances into a Vec
pub fn vec_from_pairs<'guard>(
    guard: &'guard dyn MutatorScope,
    pair_list: TaggedScopedPtr<'guard>,
) -> Result<Vec<TaggedScopedPtr<'guard>>, RuntimeError> {
    match *pair_list {
        Value::Pair(pair) => {
            let mut result = Vec::new();

            result.push(pair.first.get(guard));

            let mut next = pair.second.get(guard);
            while let Value::Pair(next_pair) = *next {
                result.push(next_pair.first.get(guard));
                next = next_pair.second.get(guard);
            }

            // we've terminated the list, but correctly?
            match *next {
                Value::Nil => Ok(result),
                _ => Err(err_eval("Incorrectly terminated Pair list")),
            }
        }
        Value::Nil => Ok(Vec::new()),
        _ => Err(err_eval("Expected a Pair")),
    }
}

/// Convenience function for unpacking a list of Pair instances into one value
pub fn value_from_1_pair<'guard>(
    guard: &'guard dyn MutatorScope,
    pair_list: TaggedScopedPtr<'guard>,
) -> Result<TaggedScopedPtr<'guard>, RuntimeError> {
    let result = vec_from_pairs(guard, pair_list)?;

    match result.as_slice() {
        [first] => Ok(*first),
        _ => Err(err_eval(&format!(
            "Pair list has {} items, expected 1",
            result.len()
        ))),
    }
}

/// Convenience function for unpacking a list of Pair instances into two values
pub fn values_from_2_pairs<'guard>(
    guard: &'guard dyn MutatorScope,
    pair_list: TaggedScopedPtr<'guard>,
) -> Result<(TaggedScopedPtr<'guard>, TaggedScopedPtr<'guard>), RuntimeError> {
    let result = vec_from_pairs(guard, pair_list)?;

    match result.as_slice() {
        [first, second] => Ok((*first, *second)),
        _ => Err(err_eval(&format!(
            "Pair list has {} items, expected 2",
            result.len()
        ))),
    }
}

/// Convenience function for unpacking a list of Pair instances into three values
pub fn values_from_3_pairs<'guard>(
    guard: &'guard dyn MutatorScope,
    pair_list: TaggedScopedPtr<'guard>,
) -> Result<
    (
        TaggedScopedPtr<'guard>,
        TaggedScopedPtr<'guard>,
        TaggedScopedPtr<'guard>,
    ),
    RuntimeError,
> {
    let result = vec_from_pairs(guard, pair_list)?;

    match result.as_slice() {
        [first, second, third] => Ok((*first, *second, *third)),
        _ => Err(err_eval(&format!(
            "Pair list has {} items, expected 3",
            result.len()
        ))),
    }
}
