use std::{
    cell::Cell,
    fmt,
    ptr::{read, write},
    slice::from_raw_parts_mut,
};

use crate::memory::{AllocObject, ArraySize};

use super::{
    containers::{
        AnyContainerFromSlice, Container, ContainerFromSlice, FillAnyContainer, FillContainer,
        IndexedAnyContainer, IndexedContainer, SliceableContainer, StackAnyContainer,
        StackContainer,
    },
    error::ErrorKind,
    printer::Print,
    rawarray::{default_array_growth, RawArray, DEFAULT_ARRAY_SIZE},
    safeptr::{MutatorScope, TaggedCellPtr, TaggedScopedPtr},
    MutatorView, RuntimeError, ScopedPtr, TypeList,
};

type BorrowFlag = isize;
const INTERIOR_ONLY: isize = 0;
const EXPOSED_MUTABLY: isize = 1;

/// An array, like Vec, but applying an interior mutability pattern.
///
/// Implements Container traits, including SliceableContainer.
/// Since SliceableContainer allows mutable access to the interior
/// of the array, RefCell-style runtime semantics are employed to
/// prevent the array being modified outside of the slice borrow.
#[derive(Clone)]
pub struct Array<T: Sized + Clone> {
    length: Cell<ArraySize>,
    data: Cell<RawArray<T>>,
    borrow: Cell<BorrowFlag>,
}

/// Internal implementation
impl<T: Sized + Clone> Array<T> {
    /// Allocate a new instance on the heap
    pub fn alloc<'guard>(
        mem: &'guard MutatorView,
    ) -> Result<ScopedPtr<'guard, Array<T>>, RuntimeError>
    where
        Array<T>: AllocObject<TypeList>,
    {
        mem.alloc(Array::new())
    }

    /// Clone the contents of an existing Array
    pub fn alloc_clone<'guard>(
        mem: &'guard MutatorView,
        from_array: ScopedPtr<'guard, Array<T>>,
    ) -> Result<ScopedPtr<'guard, Array<T>>, RuntimeError>
    where
        Array<T>: AllocObject<TypeList> + ContainerFromSlice<T>,
    {
        from_array.access_slice(mem, |items| ContainerFromSlice::from_slice(mem, items))
    }

    /// Allocate a new instance on the heap with pre-allocated capacity
    pub fn alloc_with_capacity<'guard>(
        mem: &'guard MutatorView,
        capacity: ArraySize,
    ) -> Result<ScopedPtr<'guard, Array<T>>, RuntimeError>
    where
        Array<T>: AllocObject<TypeList>,
    {
        mem.alloc(Array::with_capacity(mem, capacity)?)
    }

    /// Return a bounds-checked pointer to the object at the given index    
    fn get_offset(&self, index: ArraySize) -> Result<*mut T, RuntimeError> {
        if index >= self.length.get() {
            Err(RuntimeError::new(ErrorKind::BoundsError))
        } else {
            let ptr = self
                .data
                .get()
                .as_ptr()
                .ok_or_else(|| RuntimeError::new(ErrorKind::BoundsError))?;

            let dest_ptr = unsafe { ptr.offset(index as isize) as *mut T };

            Ok(dest_ptr)
        }
    }

    /// Bounds-checked read    
    fn read<'guard>(
        &self,
        _guard: &'guard dyn MutatorScope,
        index: ArraySize,
    ) -> Result<T, RuntimeError> {
        unsafe {
            let dest = self.get_offset(index)?;
            Ok(read(dest))
        }
    }

    /// Bounds-checked reference-read
    pub fn read_ref<'guard>(
        &self,
        _guard: &'guard dyn MutatorScope,
        index: ArraySize,
    ) -> Result<&T, RuntimeError> {
        unsafe {
            let dest = self.get_offset(index)?;
            Ok(&*dest as &T)
        }
    }

    /// Bounds-checked write    
    fn write<'guard>(
        &self,
        _guard: &'guard dyn MutatorScope,
        index: ArraySize,
        item: T,
    ) -> Result<&T, RuntimeError> {
        unsafe {
            let dest = self.get_offset(index)?;
            write(dest, item);
            Ok(&*dest as &T)
        }
    }

    /// Represent the array as a slice. This is necessarily unsafe even for the 'guard lifetime
    /// duration because while a slice is held, other code can cause array internals to change
    /// that might cause the slice pointer and length to become invalid. Interior mutability
    /// patterns such as RefCell-style should be used in addition.
    pub unsafe fn as_slice<'guard>(&self, _guard: &'guard dyn MutatorScope) -> &mut [T] {
        if let Some(ptr) = self.data.get().as_ptr() {
            from_raw_parts_mut(ptr as *mut T, self.length.get() as usize)
        } else {
            &mut []
        }
    }

    /// Represent the full capacity of the array, however initialized, as a slice.
    /// This is necessarily unsafe even for the 'guard lifetime
    /// duration because while a slice is held, other code can cause array internals to change
    /// that might cause the slice pointer and length to become invalid. Interior mutability
    /// patterns such as RefCell-style should be used in addition.
    pub unsafe fn as_capacity_slice<'guard>(&self, _guard: &'guard dyn MutatorScope) -> &mut [T] {
        if let Some(ptr) = self.data.get().as_ptr() {
            from_raw_parts_mut(ptr as *mut T, self.data.get().capacity() as usize)
        } else {
            &mut []
        }
    }
}

impl<T: Sized + Clone> Container<T> for Array<T> {
    fn new() -> Array<T> {
        Array {
            length: Cell::new(0),
            data: Cell::new(RawArray::new()),
            borrow: Cell::new(INTERIOR_ONLY),
        }
    }

    fn with_capacity<'guard>(
        mem: &'guard MutatorView,
        capacity: ArraySize,
    ) -> Result<Array<T>, RuntimeError> {
        Ok(Array {
            length: Cell::new(0),
            data: Cell::new(RawArray::with_capacity(mem, capacity)?),
            borrow: Cell::new(INTERIOR_ONLY),
        })
    }

    fn clear<'guard>(&self, _guard: &'guard MutatorView) -> Result<(), RuntimeError> {
        if self.borrow.get() != INTERIOR_ONLY {
            Err(RuntimeError::new(ErrorKind::MutableBorrowError))
        } else {
            self.length.set(0);
            Ok(())
        }
    }

    fn length(&self) -> ArraySize {
        self.length.get()
    }
}

impl<T: Sized + Clone> StackContainer<T> for Array<T> {
    /// Push can trigger an underlying array resize, hence it requires the ability to allocate    
    fn push<'guard>(&self, mem: &'guard MutatorView, item: T) -> Result<(), RuntimeError> {
        if self.borrow.get() != INTERIOR_ONLY {
            return Err(RuntimeError::new(ErrorKind::MutableBorrowError));
        }

        let length = self.length.get();
        let mut array = self.data.get(); // Takes a copy

        let capacity = array.capacity();

        if length == capacity {
            if capacity == 0 {
                array.resize(mem, DEFAULT_ARRAY_SIZE)?;
            } else {
                array.resize(mem, default_array_growth(capacity)?)?;
            }
            // Replace the struct's copy with the resized RawArray object
            self.data.set(array);
        }

        self.length.set(length + 1);
        self.write(mem, length, item)?;
        Ok(())
    }

    /// Pop returns None if the container is empty, otherwise moves the last item of the array
    /// out to the caller.
    fn pop<'guard>(&self, guard: &'guard dyn MutatorScope) -> Result<T, RuntimeError> {
        if self.borrow.get() != INTERIOR_ONLY {
            return Err(RuntimeError::new(ErrorKind::MutableBorrowError));
        }

        let length = self.length.get();

        if length == 0 {
            Err(RuntimeError::new(ErrorKind::BoundsError))
        } else {
            let last = length - 1;
            let item = self.read(guard, last)?;
            self.length.set(last);
            Ok(item)
        }
    }

    /// Return the value at the top of the stack without removing it
    fn top<'guard>(&self, guard: &'guard dyn MutatorScope) -> Result<T, RuntimeError> {
        let length = self.length.get();

        if length == 0 {
            Err(RuntimeError::new(ErrorKind::BoundsError))
        } else {
            let last = length - 1;
            let item = self.read(guard, last)?;
            Ok(item)
        }
    }
}

impl<T: Sized + Clone> IndexedContainer<T> for Array<T> {
    fn get<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        index: ArraySize,
    ) -> Result<T, RuntimeError> {
        self.read(guard, index)
    }

    fn set<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        index: ArraySize,
        item: T,
    ) -> Result<(), RuntimeError> {
        self.write(guard, index, item)?;
        Ok(())
    }
}

impl<T: Clone + Sized> ContainerFromSlice<T> for Array<T>
where
    Array<T>: AllocObject<TypeList>,
{
    fn from_slice<'guard>(
        mem: &'guard MutatorView,
        data: &[T],
    ) -> Result<ScopedPtr<'guard, Array<T>>, RuntimeError> {
        let array = Array::alloc_with_capacity(mem, data.len() as ArraySize)?;
        let slice = unsafe { array.as_capacity_slice(mem) };
        slice.clone_from_slice(data);
        array.length.set(data.len() as ArraySize);
        Ok(array)
    }
}

impl StackAnyContainer for Array<TaggedCellPtr> {
    /// Push can trigger an underlying array resize, hence it requires the ability to allocate    
    fn push<'guard>(
        &self,
        mem: &'guard MutatorView,
        item: TaggedScopedPtr<'guard>,
    ) -> Result<(), RuntimeError> {
        StackContainer::<TaggedCellPtr>::push(self, mem, TaggedCellPtr::new_with(item))
    }

    /// Pop returns None if the container is empty, otherwise moves the last item of the array
    /// out to the caller.
    fn pop<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
    ) -> Result<TaggedScopedPtr<'guard>, RuntimeError> {
        Ok(StackContainer::<TaggedCellPtr>::pop(self, guard)?.get(guard))
    }

    /// Return the value at the top of the stack without removing it
    fn top<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
    ) -> Result<TaggedScopedPtr<'guard>, RuntimeError> {
        Ok(StackContainer::<TaggedCellPtr>::top(self, guard)?.get(guard))
    }
}

impl<T: Sized + Clone> SliceableContainer<T> for Array<T> {
    fn access_slice<'guard, F, R>(&self, guard: &'guard dyn MutatorScope, f: F) -> R
    where
        F: FnOnce(&mut [T]) -> R,
    {
        self.borrow.set(EXPOSED_MUTABLY);
        let slice = unsafe { self.as_slice(guard) };
        let result = f(slice);
        self.borrow.set(INTERIOR_ONLY);
        result
    }
}

impl AnyContainerFromSlice for Array<TaggedCellPtr> {
    fn from_slice<'guard>(
        mem: &'guard MutatorView,
        data: &[TaggedScopedPtr<'guard>],
    ) -> Result<ScopedPtr<'guard, Self>, RuntimeError> {
        let array = Array::<TaggedCellPtr>::alloc_with_capacity(mem, data.len() as ArraySize)?;
        let slice = unsafe { array.as_capacity_slice(mem) };

        // probably slow
        for index in 0..data.len() {
            slice[index] = TaggedCellPtr::new_with(data[index])
        }

        array.length.set(data.len() as ArraySize);
        Ok(array)
    }
}

/// Array of u8
pub type ArrayU8 = Array<u8>;
/// Array of u16
pub type ArrayU16 = Array<u16>;
/// Array of u32
pub type ArrayU32 = Array<u32>;

impl Print for ArrayU8 {
    fn print<'guard>(
        &self,
        _guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "ArrayU8[...]")
    }
}

impl Print for ArrayU16 {
    fn print<'guard>(
        &self,
        _guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "ArrayU16[...]")
    }
}

impl Print for ArrayU32 {
    fn print<'guard>(
        &self,
        _guard: &'guard dyn MutatorScope,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "ArrayU32[...]")
    }
}

impl IndexedAnyContainer for Array<TaggedCellPtr> {
    /// Return a pointer to the object at the given index. Bounds-checked.
    fn get<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        index: ArraySize,
    ) -> Result<TaggedScopedPtr<'guard>, RuntimeError> {
        Ok(self.read_ref(guard, index)?.get(guard))
    }

    /// Set the object pointer at the given index. Bounds-checked.
    fn set<'guard>(
        &self,
        guard: &'guard dyn MutatorScope,
        index: ArraySize,
        item: TaggedScopedPtr<'guard>,
    ) -> Result<(), RuntimeError> {
        self.read_ref(guard, index)?.set(item);
        Ok(())
    }
}

impl<T: Sized + Clone> FillContainer<T> for Array<T> {
    fn fill<'guard>(
        &self,
        mem: &'guard MutatorView,
        size: ArraySize,
        item: T,
    ) -> Result<(), RuntimeError> {
        let length = self.length();

        if length > size {
            Ok(())
        } else {
            let mut array = self.data.get(); // Takes a copy

            let capacity = array.capacity();

            if size > capacity {
                if capacity == 0 {
                    array.resize(mem, DEFAULT_ARRAY_SIZE)?;
                } else {
                    array.resize(mem, default_array_growth(capacity)?)?;
                }
                // Replace the struct's copy with the resized RawArray object
                self.data.set(array);
            }

            self.length.set(size);

            for index in length..size {
                self.write(mem, index, item.clone())?;
            }

            Ok(())
        }
    }
}

impl FillAnyContainer for Array<TaggedCellPtr> {
    fn fill<'guard>(
        &self,
        mem: &'guard MutatorView,
        size: ArraySize,
        item: TaggedScopedPtr<'guard>,
    ) -> Result<(), RuntimeError> {
        let length = self.length();

        if length > size {
            Ok(())
        } else {
            let mut array = self.data.get(); // Takes a copy

            let capacity = array.capacity();

            if size > capacity {
                if capacity == 0 {
                    array.resize(mem, DEFAULT_ARRAY_SIZE)?;
                } else {
                    array.resize(mem, default_array_growth(capacity)?)?;
                }
                // Replace the struct's copy with the resized RawArray object
                self.data.set(array);
            }

            self.length.set(size);

            for index in length..size {
                self.write(mem, index, TaggedCellPtr::new_with(item))?;
            }

            Ok(())
        }
    }
}
