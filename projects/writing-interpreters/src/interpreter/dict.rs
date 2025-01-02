use std::{cell::Cell, fmt, hash::Hasher};

use fnv::FnvHasher;

use crate::memory::ArraySize;

use super::{
    containers::HashIndexedAnyContainer,
    error::ErrorKind,
    hashable::Hashable,
    printer::Print,
    rawarray::{default_array_growth, RawArray},
    safeptr::{MutatorScope, TaggedCellPtr, TaggedScopedPtr},
    taggedptr::Value,
    MutatorView, RuntimeError,
};

// max load factor before resizing the table
const LOAD_FACTOR: f32 = 0.80;
const TOMBSTONE: u64 = 1;

/// A mutable Dict key/value associative data structure.
pub struct Dict {
    /// Number of items stored
    length: Cell<ArraySize>,
    /// Total count of items plus tombstones
    used_entries: Cell<ArraySize>,
    /// Backing array for key/value entries
    data: Cell<RawArray<DictItem>>,
}

/// Internal entry representation, keeping copy of hash for the key
#[derive(Clone)]
pub struct DictItem {
    key: TaggedCellPtr,
    value: TaggedCellPtr,
    hash: u64,
}

impl Dict {
    /// Scale capacity up if needed
    fn grow_capacity<'guard>(&self, mem: &'guard MutatorView) -> Result<(), RuntimeError> {
        let data = self.data.get();

        let new_capacity = default_array_growth(data.capacity())?;
        let new_data = RawArray::<DictItem>::with_capacity(mem, new_capacity)?;

        let maybe_ptr = data.as_ptr();
        if let Some(ptr) = maybe_ptr {
            for index in 0..data.capacity() {
                let entry =
                    unsafe { &mut *(ptr.offset(index as isize) as *mut DictItem) as &mut DictItem };
                if !entry.key.is_nil() {
                    let new_entry = find_entry(mem, &new_data, entry.hash)?;
                    *new_entry = entry.clone();
                }
            }
        }

        self.data.set(new_data);
        Ok(())
    }
}

/// Generate a hash value for a key
fn hash_key<'guard>(
    guard: &'guard dyn MutatorScope,
    key: TaggedScopedPtr<'guard>,
) -> Result<u64, RuntimeError> {
    match *key {
        Value::Symbol(sym) => {
            let mut hasher = FnvHasher::default();
            sym.hash(guard, &mut hasher);
            Ok(hasher.finish())
        }
        Value::Number(n) => Ok(n as u64),
        _ => Err(RuntimeError::new(ErrorKind::UnhashableError)),
    }
}

/// Given a key, generate the hash and search for an entry that either matches this hash
/// or the next available blank entry.
fn find_entry<'guard>(
    _guard: &'guard dyn MutatorScope,
    data: &RawArray<DictItem>,
    hash: u64,
) -> Result<&'guard mut DictItem, RuntimeError> {
    // get raw pointer to base of array
    let ptr = data
        .as_ptr()
        .ok_or(RuntimeError::new(ErrorKind::BoundsError))?;

    // calculate the starting index into `data` to begin scanning at
    let mut index = (hash % data.capacity() as u64) as ArraySize;
    // the first tombstone we find will be saved here
    let mut tombstone: Option<&mut DictItem> = None;

    loop {
        let entry = unsafe { &mut *(ptr.offset(index as isize) as *mut DictItem) as &mut DictItem };
        if entry.hash == TOMBSTONE && entry.key.is_nil() {
            // this is a tombstone: save the first tombstone reference we find
            if tombstone.is_none() {
                // Keep tombstone for now in case we find an exact match later
                tombstone = Some(entry);
            }
        } else if entry.hash == hash {
            // this is an exact match slot
            return Ok(entry);
        } else if entry.key.is_nil() {
            // this is a non-tombstone empty slot
            if let Some(earlier_entry) = tombstone {
                // we found a tombstone, so we can use it
                return Ok(earlier_entry);
            } else {
                // we found an empty slot. No tombstone earlier
                return Ok(entry);
            }
        }

        // increment the index, wrapping back to 0 when we get to the end of the array
        index = (index + 1) % data.capacity();
    }
}

/// Returns true if the dict has reached it's defined load factor and needs to be resized before inserting
/// a new entry.
fn needs_to_grow(used_entries: ArraySize, capacity: ArraySize) -> bool {
    let ratio = (used_entries as f32) / (capacity as f32);
    ratio > LOAD_FACTOR
}

/// Hashable-indexed interface. Objects used as keys must implement Hashable.
impl HashIndexedAnyContainer for Dict {
    fn lookup<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        key: TaggedScopedPtr,
    ) -> Result<TaggedScopedPtr<'guard>, RuntimeError> {
        let hash = hash_key(guard, key)?;
        let data = self.data.get();
        let entry = find_entry(guard, &data, hash)?;

        if entry.key.is_nil() {
            // a nil key means the key was not found in the Dict
            Err(RuntimeError::new(ErrorKind::KeyError))
        } else {
            Ok(entry.value.get(guard))
        }
    }

    fn assoc<'guard>(
        &self,
        mem: &'guard super::MutatorView,
        key: TaggedScopedPtr<'guard>,
        value: TaggedScopedPtr<'guard>,
    ) -> Result<(), RuntimeError> {
        let hash = hash_key(mem, key)?;

        let mut data = self.data.get();
        if needs_to_grow(self.used_entries.get() + 1, data.capacity()) {
            self.grow_capacity(mem)?;
            data = self.data.get();
        }

        let entry = find_entry(mem, &data, hash)?;
        if entry.key.is_nil() {
            self.length.set(self.length.get() + 1);
            if entry.hash == 0 {
                // not a tombstone
                self.used_entries.set(self.used_entries.get() + 1);
            }
        }

        entry.key.set(key);
        entry.value.set(value);
        entry.hash = hash;

        Ok(())
    }

    fn dissoc<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        key: TaggedScopedPtr,
    ) -> Result<TaggedScopedPtr<'guard>, RuntimeError> {
        let hash = hash_key(guard, key)?;

        let data = self.data.get();
        let entry = find_entry(guard, &data, hash)?;

        if entry.key.is_nil() {
            // a nil key means the key was not found in the Dict
            return Err(RuntimeError::new(ErrorKind::KeyError));
        }

        self.length.set(self.length.get() - 1);

        entry.key.set_to_nil();
        entry.hash = TOMBSTONE;

        Ok(entry.value.get(guard))
    }

    fn exists<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        key: TaggedScopedPtr,
    ) -> Result<bool, RuntimeError> {
        let hash = hash_key(guard, key)?;
        let data = self.data.get();
        let entry = find_entry(guard, &data, hash)?;

        Ok(!entry.key.is_nil())
    }
}

impl Print for Dict {
    fn print<'guard>(
        &self,
        _guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "Dict[unimplemented]")
    }
}
