use std::mem::size_of;
use std::ptr::{write, NonNull};
use std::slice::from_raw_parts_mut;
use std::{cell::UnsafeCell, marker::PhantomData, mem::replace};

use crate::memory::stickyimmix::BLOCK_CAPACITY;

use super::allocator::{alloc_size_of, ArraySize};
use super::{
    allocator::{AllocHeader, AllocRaw},
    AllocError, BumpBlock, SizeClass,
};
use super::{Mark, RawPtr};

pub struct StickyimmixHeap<H> {
    blocks: UnsafeCell<BlockList>,
    _header_type: PhantomData<*const H>,
}

impl<H> StickyimmixHeap<H> {
    pub fn new() -> Self {
        StickyimmixHeap {
            blocks: UnsafeCell::new(BlockList::new()),
            _header_type: PhantomData,
        }
    }

    fn find_space(
        &self,
        alloc_size: usize,
        size_class: SizeClass,
    ) -> Result<*const u8, AllocError> {
        if size_class == SizeClass::Large {
            return Err(AllocError::BadRequest);
        }

        let blocks = unsafe { &mut *self.blocks.get() };
        let space = match blocks.head {
            Some(ref mut head) => {
                if size_class == SizeClass::Medium && alloc_size > head.current_hole_size() {
                    return blocks.overflow_alloc(alloc_size);
                }

                match head.inner_alloc(alloc_size) {
                    // the block has a suitable hole
                    Some(space) => space,
                    None => {
                        let previous = replace(head, BumpBlock::new()?);
                        blocks.rest.push(previous);
                        head.inner_alloc(alloc_size).expect("Unexpected error!")
                    }
                }
            }
            None => {
                let mut head = BumpBlock::new()?;
                let space = head.inner_alloc(alloc_size).expect("Unexpected error!");
                blocks.head = Some(head);
                space
            }
        };

        Ok(space)
    }
}

impl<H: AllocHeader> AllocRaw for StickyimmixHeap<H> {
    type Header = H;

    fn alloc<T>(&self, object: T) -> Result<super::RawPtr<T>, AllocError>
    where
        T: super::allocator::AllocObject<<Self::Header as AllocHeader>::TypeId>,
    {
        // calculate the total size of the object and it's header
        let header_size = size_of::<Self::Header>();
        let object_size = size_of::<T>();
        let total_size = header_size + object_size;

        // round the size to the next word boundary to keep objects aligned and get the size class
        let alloc_size = alloc_size_of(total_size);
        let size_class = SizeClass::get_for_size(alloc_size)?;

        let space = self.find_space(alloc_size, size_class)?;
        let header = Self::Header::new::<T>(object_size as ArraySize, size_class, Mark::Allocated);

        unsafe {
            write(space as *mut Self::Header, header);
        }

        let object_space = unsafe { space.offset(header_size as isize) };
        unsafe {
            write(object_space as *mut T, object);
        }

        Ok(RawPtr::new(object_space as *const T))
    }

    fn alloc_array(
        &self,
        size_bytes: super::allocator::ArraySize,
    ) -> Result<super::RawPtr<u8>, AllocError> {
        let header_size = size_of::<Self::Header>();
        let total_size = header_size + size_bytes as usize;

        let alloc_size = alloc_size_of(total_size);
        let size_class = SizeClass::get_for_size(alloc_size)?;

        let space = self.find_space(alloc_size, size_class)?;

        let header = Self::Header::new_array(size_bytes, size_class, Mark::Allocated);

        unsafe {
            write(space as *mut Self::Header, header);
        }

        let array_space = unsafe { space.offset(header_size as isize) };

        // Initialize object_space to zero here.
        // If using the system allocator for any objects (SizeClass::Large, for example),
        // the memory may already be zeroed.
        let array = unsafe { from_raw_parts_mut(array_space as *mut u8, size_bytes as usize) };
        // The compiler should recognize this as optimizable
        for byte in array {
            *byte = 0;
        }

        Ok(RawPtr::new(array_space as *const u8))
    }

    fn get_header(object: std::ptr::NonNull<()>) -> std::ptr::NonNull<Self::Header> {
        unsafe { NonNull::new_unchecked(object.cast::<Self::Header>().as_ptr().offset(-1)) }
    }

    fn get_object(header: std::ptr::NonNull<Self::Header>) -> std::ptr::NonNull<()> {
        unsafe { NonNull::new_unchecked(header.as_ptr().offset(1).cast::<()>()) }
    }
}

/// A list of blocks as the current block being allocated into and a list
/// of full blocks
struct BlockList {
    /// the current block being allocated into
    head: Option<BumpBlock>,
    /// a block kept handy for writing medium objects into that don't fit the head block's current hole
    overflow: Option<BumpBlock>,
    /// allocated into but are not suitable for recycling
    rest: Vec<BumpBlock>,
}

impl BlockList {
    fn new() -> BlockList {
        BlockList {
            head: None,
            overflow: None,
            rest: Vec::new(),
        }
    }

    /// Allocate a space for a medium object into an overflow block
    fn overflow_alloc(&mut self, alloc_size: usize) -> Result<*const u8, AllocError> {
        assert!(alloc_size <= BLOCK_CAPACITY);
        let space = match self.overflow {
            Some(ref mut overflow) => match overflow.inner_alloc(alloc_size) {
                // the block has a suitable hole
                Some(space) => space,
                None => {
                    let previous = replace(overflow, BumpBlock::new()?);
                    self.rest.push(previous);
                    overflow.inner_alloc(alloc_size).expect("Unexpected error!")
                }
            },
            None => {
                let mut overflow = BumpBlock::new()?;
                let space = overflow
                    .inner_alloc(alloc_size)
                    .expect("We expected this object to fit!");

                self.overflow = Some(overflow);

                space
            }
        };

        Ok(space)
    }
}
