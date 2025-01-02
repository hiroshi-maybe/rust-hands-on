use crate::memory::{AllocHeader, AllocObject, AllocRaw, AllocTypeId, Mark, RawPtr, SizeClass};

use super::{
    bytecode::ByteCode,
    dict::Dict,
    function::{Function, Partial},
    list::List,
    memory::HeapStorage,
    number::NumberObject,
    pair::Pair,
    pointerops::{AsNonNull, Tagged},
    symbol::Symbol,
    taggedptr::FatPtr,
    text::Text,
    vm::Upvalue,
    ArrayU16, ArrayU32, ArrayU8,
};

/// Recognized heap-allocated types.
/// This should represent every type native to the runtime with the exception of tagged pointer inline value
/// types.
#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TypeList {
    ArrayBackingBytes,
    ArrayOpcode,
    ArrayU8,
    ArrayU16,
    ArrayU32,
    ByteCode,
    CallFrameList,
    Dict,
    Function,
    InstructionStream,
    List,
    NumberObject,
    Pair,
    Partial,
    Symbol,
    Text,
    Thread,
    Upvalue,
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

impl ObjectHeader {
    /// Convert the ObjectHeader address to a FatPtr pointing at the object itself.
    // NOTE Any type that is a runtime dynamic type must be added to the below list
    // NOTE Be careful to match the correct TypeList discriminant with it's corresponding FatPtr discriminant
    // NOTE Be careful to untag the pointer before putting it into a `FatPtr`
    // ANCHOR: DefObjectHeaderGetObjectFatPtr
    pub unsafe fn get_object_fatptr(&self) -> FatPtr {
        let ptr_to_self = self.non_null_ptr();
        let object_addr = HeapStorage::get_object(ptr_to_self);

        match self.type_id {
            TypeList::ArrayU8 => FatPtr::ArrayU8(RawPtr::untag(object_addr.cast::<ArrayU8>())),
            TypeList::ArrayU16 => FatPtr::ArrayU16(RawPtr::untag(object_addr.cast::<ArrayU16>())),
            TypeList::ArrayU32 => FatPtr::ArrayU32(RawPtr::untag(object_addr.cast::<ArrayU32>())),
            TypeList::Dict => FatPtr::Dict(RawPtr::untag(object_addr.cast::<Dict>())),
            TypeList::Function => FatPtr::Function(RawPtr::untag(object_addr.cast::<Function>())),
            TypeList::List => FatPtr::List(RawPtr::untag(object_addr.cast::<List>())),
            TypeList::NumberObject => {
                FatPtr::NumberObject(RawPtr::untag(object_addr.cast::<NumberObject>()))
            }
            TypeList::Pair => FatPtr::Pair(RawPtr::untag(object_addr.cast::<Pair>())),
            TypeList::Partial => FatPtr::Partial(RawPtr::untag(object_addr.cast::<Partial>())),
            TypeList::Symbol => FatPtr::Symbol(RawPtr::untag(object_addr.cast::<Symbol>())),
            TypeList::Text => FatPtr::Text(RawPtr::untag(object_addr.cast::<Text>())),
            TypeList::Upvalue => FatPtr::Upvalue(RawPtr::untag(object_addr.cast::<Upvalue>())),

            // Other types not represented by FatPtr are an error to id here
            _ => panic!("Invalid ObjectHeader type tag {:?}!", self.type_id),
        }
    }
}
impl AsNonNull for ObjectHeader {}

impl AllocHeader for ObjectHeader {
    type TypeId = TypeList;

    fn new<O: crate::memory::allocator::AllocObject<Self::TypeId>>(
        size: u32,
        size_class: SizeClass,
        mark: Mark,
    ) -> Self {
        ObjectHeader {
            mark,
            size_class,
            type_id: O::TYPE_ID,
            size_bytes: size,
        }
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

/// Apply the type ID to each native type
macro_rules! declare_allocobject {
    ($T:ty, $I:tt) => {
        impl AllocObject<TypeList> for $T {
            const TYPE_ID: TypeList = TypeList::$I;
        }
    };
}

declare_allocobject!(ByteCode, ByteCode);
declare_allocobject!(Pair, Pair);
declare_allocobject!(Symbol, Symbol);
