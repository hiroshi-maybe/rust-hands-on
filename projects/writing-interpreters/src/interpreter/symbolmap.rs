use std::{cell::RefCell, collections::HashMap};

use super::{arena::Arena, symbol::Symbol};
use crate::memory::{AllocRaw, RawPtr};

/// A mapping of symbol names (Strings) to Symbol pointers. Only one copy of the symbol
/// name String is kept; a Symbol resides in managed memory with a raw pointer to the
/// String. Thus the lifetime of the SymbolMap must be at least the lifetime of the
/// managed memory. This is arranged here by maintaining Symbol memory alongside the
/// mapping HashMap.
///
/// No Symbol is ever deleted. Symbol name strings must be immutable.
pub struct SymbolMap {
    map: RefCell<HashMap<String, RawPtr<Symbol>>>,
    arena: Arena,
}

impl SymbolMap {
    pub fn lookup(&self, name: &str) -> RawPtr<Symbol> {
        {
            if let Some(ptr) = self.map.borrow().get(name) {
                return *ptr;
            }
        }

        let name = String::from(name);
        let ptr = self.arena.alloc(Symbol::new(&name)).unwrap();
        self.map.borrow_mut().insert(name, ptr);
        ptr
    }
}
