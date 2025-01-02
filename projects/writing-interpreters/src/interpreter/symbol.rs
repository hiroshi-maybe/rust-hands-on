use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::slice;
use std::str;

use super::hashable::Hashable;
use super::printer::Print;
use super::safeptr::MutatorScope;

/// A Symbol is a unique object that has a unique name string. The backing storage for the
/// underlying str data must have a lifetime of at least that of the Symbol instance to
/// prevent use-after-free.
/// See `SymbolMap`
#[derive(Copy, Clone)]
pub struct Symbol {
    name_ptr: *const u8,
    name_len: usize,
}

impl Symbol {
    pub fn new(name: &str) -> Self {
        Symbol {
            name_ptr: name.as_ptr(),
            name_len: name.len(),
        }
    }

    pub unsafe fn unguarded_as_str<'desired_lifetime>(&self) -> &'desired_lifetime str {
        let slice = slice::from_raw_parts(self.name_ptr, self.name_len);
        str::from_utf8(slice).unwrap()
    }

    pub fn as_str<'guard>(&self, _guard: &'guard dyn MutatorScope) -> &'guard str {
        unsafe { &self.unguarded_as_str() }
    }
}

impl Hashable for Symbol {
    fn hash<'guard, H: Hasher>(&self, guard: &'guard dyn MutatorScope, h: &mut H) {
        self.as_str(guard).hash(h)
    }
}

impl Print for Symbol {
    /// Safe because the lifetime of `MutatorScope` defines a safe-access window
    fn print<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "{}", self.as_str(guard))
    }
}
