use std::hash::Hasher;

use super::safeptr::MutatorScope;

/// Similar to Hash but for use in a mutator lifetime-limited scope
pub trait Hashable {
    fn hash<'guard, H: Hasher>(&self, _guard: &'guard dyn MutatorScope, hasher: &mut H);
}
