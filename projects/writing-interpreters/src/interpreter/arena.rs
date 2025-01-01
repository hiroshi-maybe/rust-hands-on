use std::ptr::NonNull;

use crate::memory::{
    AllocError, AllocHeader, AllocObject, AllocRaw, ArraySize, Mark, RawPtr, SizeClass,
    StickyImmixHeap,
};

use super::{RuntimeError, TypeList};

/// Allocation header for an Arena-allocated value
pub struct ArenaHeader {}

impl AllocHeader for ArenaHeader {
    type TypeId = TypeList;

    fn new<O: crate::memory::allocator::AllocObject<Self::TypeId>>(
        size: u32,
        size_class: SizeClass,
        mark: Mark,
    ) -> Self {
        ArenaHeader {}
    }

    fn new_array(
        size: crate::memory::allocator::ArraySize,
        size_class: SizeClass,
        mark: Mark,
    ) -> Self {
        ArenaHeader {}
    }

    fn mark(&mut self) {}

    fn is_marked(&self) -> bool {
        true
    }

    fn size_class(&self) -> SizeClass {
        SizeClass::Small
    }

    fn size(&self) -> u32 {
        1
    }

    fn type_id(&self) -> Self::TypeId {
        TypeList::Symbol
    }
}

/// A non-garbage-collected pool of memory blocks for interned values.
/// These values are not dropped on Arena deallocation.
/// Values must be "atomic", that is, not composed of other object
/// pointers that need to be traced.
///
/// Since symbols are unique strings that can be identified and compared
/// by their pointer values, these pointer values must remain static
/// throughout the program lifetime. Thus, Symbol objects cannot be managed
/// by a heap that might perform object relocation. We need a separate
/// heap type for objects that are never moved or freed unil the program ends,
/// the Arena type.
pub struct Arena {
    heap: StickyImmixHeap<ArenaHeader>,
}

impl Arena {
    pub fn new() -> Arena {
        Arena {
            heap: StickyImmixHeap::new(),
        }
    }
}

impl AllocRaw for Arena {
    type Header = ArenaHeader;

    fn alloc<T>(&self, object: T) -> Result<RawPtr<T>, AllocError>
    where
        T: AllocObject<TypeList>,
    {
        self.heap.alloc(object)
    }

    fn alloc_array(&self, _size_bytes: ArraySize) -> Result<RawPtr<u8>, AllocError> {
        unimplemented!()
    }

    fn get_header(_object: NonNull<()>) -> NonNull<Self::Header> {
        unimplemented!()
    }

    fn get_object(_header: NonNull<Self::Header>) -> NonNull<()> {
        unimplemented!()
    }
}
