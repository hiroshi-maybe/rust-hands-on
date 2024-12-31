use std::{cell::UnsafeCell, marker::PhantomData, mem::replace};

use crate::memory::stickyimmix::BLOCK_CAPACITY;

use super::stickyimmix::{AllocError, BumpBlock};

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

/// Object size class.
/// - Small objects fit inside a line
/// - Medium objects span more than one line
/// - Large objects span multiple blocks
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SizeClass {
    Small,
    Medium,
    Large,
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
