use std::{cell::Cell, fmt, ptr::read, ptr::write};

use crate::memory::{AllocObject, ArraySize};

use super::{
    containers::{Container, StackContainer},
    error::ErrorKind,
    printer::Print,
    rawarray::{default_array_growth, RawArray, DEFAULT_ARRAY_SIZE},
    safeptr::MutatorScope,
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
