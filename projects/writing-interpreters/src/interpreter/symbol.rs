use std::slice;
use std::str;

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
