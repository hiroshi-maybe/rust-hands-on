use std::fmt;

use super::{
    bytecode::ByteCode,
    list::List,
    printer::Print,
    safeptr::{MutatorScope, TaggedCellPtr},
    taggedptr::Value,
    ArrayU16, CellPtr, ScopedPtr,
};

/// A function object type
#[derive(Clone)]
pub struct Function {
    /// name could be a Symbol, or nil if it is an anonymous fn
    name: TaggedCellPtr,
    /// Number of arguments required to activate the function
    arity: u8,
    /// Instructions comprising the function code
    code: CellPtr<ByteCode>,
    /// Param names are stored for introspection of a function signature
    param_names: CellPtr<List>,
    /// List of (CallFrame-index: u8 | Window-index: u8) relative offsets from this function's
    /// declaration where nonlocal variables will be found. Needed when creating a closure. May be
    /// nil
    nonlocal_refs: TaggedCellPtr,
}

impl Function {
    /// Return a list of nonlocal stack references referenced by the function. It is a panickable
    /// offense to call this when there are no nonlocals referenced by the function. This would
    /// indicate a compiler bug.
    pub fn nonlocals<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
    ) -> ScopedPtr<'guard, ArrayU16> {
        match *self.nonlocal_refs.get(guard) {
            Value::ArrayU16(nonlocals) => nonlocals,
            _ => unreachable!(),
        }
    }
}

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
pub struct Partial {
    /// Remaining number of arguments required to activate the function
    arity: u8,
    /// Number of arguments already applied
    used: u8,
    /// List of argument values already applied
    args: CellPtr<List>,
    /// Closure environment - must be either nil or a List of Upvalues
    env: TaggedCellPtr,
    /// Function that will be activated when all arguments are applied
    func: CellPtr<Function>,
}

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
