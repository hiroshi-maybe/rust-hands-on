use std::fmt;

use super::{printer::Print, safeptr::MutatorScope};

/// While Text is somewhat similar to Symbol, it is instead garbage-collected heap allocated and not interned.
#[derive(Copy, Clone)]
pub struct Text {}

impl Print for Text {
    fn print<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "\"(unimplemented)\"")
    }
}
