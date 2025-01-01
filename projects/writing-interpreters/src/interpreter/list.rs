use std::fmt;

use super::{
    array::Array,
    printer::Print,
    safeptr::{MutatorScope, TaggedCellPtr},
};

/// A List can contain a mixed sequence of any type of value
pub type List = Array<TaggedCellPtr>;

impl Print for List {
    fn print<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "(List unimplemented)")
    }
}
