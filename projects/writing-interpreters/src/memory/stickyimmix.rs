use std::mem::size_of;

use super::Block;

pub const BLOCK_SIZE_BITS: usize = 15;
pub const BLOCK_SIZE: usize = 1 << BLOCK_SIZE_BITS;

pub struct BumpBlock {
    // bump pointer. The index into the block where the last object was written
    cursor: *const u8,
    limit: *const u8,
    // Block in which objects will be written
    block: Block,
    meta: BlockMeta,
}

pub struct BlockMeta {
    lines: *mut u8,
}

impl BumpBlock {
    pub fn inner_alloc(&mut self, alloc_size: usize) -> Option<*const u8> {
        // Allocated downwards
        let cursor_ptr = self.cursor as usize;
        let align_mask: usize = !(size_of::<usize>() - 1);
        let next_ptr = cursor_ptr.checked_sub(alloc_size)? & align_mask;

        let block_start_ptr = self.block.as_ptr() as usize;
        if next_ptr < block_start_ptr {
            // The current block does not have enough capacity for the `alloc_size`
            None
        } else {
            self.cursor = next_ptr as *const u8;
            Some(next_ptr as *const u8)
        }
    }
}
