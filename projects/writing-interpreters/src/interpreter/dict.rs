use std::fmt;

use super::{printer::Print, safeptr::MutatorScope};

/// A mutable Dict key/value associative data structure.
pub struct Dict {}

impl Print for Dict {
    fn print<'guard>(
        &self,
        _guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "Dict[unimplemented]")
    }
}
