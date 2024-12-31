use crate::memory::{AllocHeader, AllocTypeId, Mark, SizeClass};

/// Recognized heap-allocated types.
/// This should represent every type native to the runtime with the exception of tagged pointer inline value
/// types.
#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TypeList {
    Text,
}

// Mark this as a Stickyimmix type-identifier type
impl AllocTypeId for TypeList {}

/// A heap-allocated object header
pub struct ObjectHeader {
    mark: Mark,
    size_class: SizeClass,
    type_id: TypeList,
    size_bytes: u32,
}

impl AllocHeader for ObjectHeader {
    type TypeId = TypeList;

    fn new<O: crate::memory::allocator::AllocObject<Self::TypeId>>(
        size: u32,
        size_class: SizeClass,
        mark: Mark,
    ) -> Self {
        todo!()
    }

    fn new_array(
        size: crate::memory::allocator::ArraySize,
        size_class: SizeClass,
        mark: Mark,
    ) -> Self {
        todo!()
    }

    fn mark(&mut self) {
        todo!()
    }

    fn is_marked(&self) -> bool {
        todo!()
    }

    fn size_class(&self) -> SizeClass {
        todo!()
    }

    fn size(&self) -> u32 {
        todo!()
    }

    fn type_id(&self) -> Self::TypeId {
        todo!()
    }
}
