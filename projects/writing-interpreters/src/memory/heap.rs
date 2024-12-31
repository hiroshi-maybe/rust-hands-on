use std::mem::replace;

use crate::memory::stickyimmix::BLOCK_CAPACITY;

use super::stickyimmix::{AllocError, BumpBlock};

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
