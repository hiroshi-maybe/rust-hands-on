use std::fmt;

use super::{printer::Print, safeptr::MutatorScope};

/// A function object type
#[derive(Clone)]
pub struct Function {}

impl Print for Function {
    /// Prints a string representation of the function
    fn print<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "(Function unimplemented)")
    }

    /// Prints the disassembled bytecode
    fn debug<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "(Function unimplemented)")
    }
}

/// A partial function application object type
#[derive(Clone)]
pub struct Partial {}

impl Print for Partial {
    /// Prints a string representation of the Partial object
    fn print<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "(Patrial function unimplemented)")
    }

    /// Prints the associated function's disassembled bytecode
    fn debug<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "(Partial function unimplemented)")
    }
}
