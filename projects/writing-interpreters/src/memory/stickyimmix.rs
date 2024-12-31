use std::mem::size_of;

use super::{block::BlockError, Block};

pub const BLOCK_SIZE_BITS: usize = 15;
pub const BLOCK_SIZE: usize = 1 << BLOCK_SIZE_BITS; // 32KB

pub const LINE_SIZE_BITS: usize = 7;
pub const LINE_SIZE: usize = 1 << LINE_SIZE_BITS; // 128 bytes
pub const LINE_COUNT: usize = BLOCK_SIZE / LINE_SIZE; // 256 lines per block

pub const BLOCK_CAPACITY: usize = BLOCK_SIZE - LINE_COUNT; // 32KB - 256B
/// The first line-mark offset into the block is here.
pub const LINE_MARK_START: usize = BLOCK_CAPACITY;

pub struct BumpBlock {
    /// bump pointer. The index into the block where the last object was written
    cursor: *const u8,
    /// known byte offset limit into which we can allocate
    limit: *const u8,
    /// Block in which objects will be written
    block: Block,
    /// Metdata for the block
    meta: BlockMeta,
}

pub struct BlockMeta {
    lines: *mut u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AllocError {
    /// Some attribute of the allocation, most likely the size requested,
    /// could not be fulfilled
    BadRequest,
    /// Out of memory - allocating the space failed
    OOM,
}

impl BumpBlock {
    /// Create a new block of heap space and it's metadata, placing a
    /// pointer to the metadata in the first word of the block.
    pub fn new() -> Result<BumpBlock, AllocError> {
        let inner_block = Block::new(BLOCK_SIZE)?;
        let block_ptr = inner_block.as_ptr();

        let block = BumpBlock {
            cursor: unsafe { block_ptr.add(BLOCK_CAPACITY) },
            limit: block_ptr,
            block: inner_block,
            meta: BlockMeta::new(block_ptr),
        };

        Ok(block)
    }

    pub fn inner_alloc(&mut self, alloc_size: usize) -> Option<*const u8> {
        // Allocated downwards
        let cursor_ptr = self.cursor as usize;
        let align_mask: usize = !(size_of::<usize>() - 1);
        let next_ptr = cursor_ptr.checked_sub(alloc_size)? & align_mask;

        let limit = self.limit as usize;
        let block_start_ptr = self.block.as_ptr() as usize;
        if next_ptr < limit {
            let next_starting_at = unsafe { self.limit.sub(block_start_ptr) as usize } as usize;

            if next_starting_at > 0 {
                if let Some((cursor, limit)) = self
                    .meta
                    .find_next_available_hole(next_starting_at, alloc_size)
                {
                    self.cursor = unsafe { self.block.as_ptr().add(cursor) };
                    self.limit = unsafe { self.block.as_ptr().add(limit) };
                    return self.inner_alloc(alloc_size);
                }
            }

            None
        } else {
            self.cursor = next_ptr as *const u8;
            Some(next_ptr as *const u8)
        }
    }
}

impl BlockMeta {
    /// Heap allocate a metadata instance so that it doesn't move so we can store pointers
    /// to it.
    pub fn new(block_ptr: *const u8) -> BlockMeta {
        let mut meta = BlockMeta {
            lines: unsafe { block_ptr.add(LINE_MARK_START) as *mut u8 },
        };
        meta.reset();

        meta
    }

    /// Mark the indexed line
    pub fn mark_line(&mut self, index: usize) {
        unsafe { *self.as_line_mark(index) = 1 };
    }

    // locate a gap of unmarked lines of sufficient size
    pub fn find_next_available_hole(
        &self,
        starting_at: usize,
        alloc_size: usize,
    ) -> Option<(usize, usize)> {
        let starting_line = starting_at / LINE_SIZE;
        let lines_required = (alloc_size + LINE_SIZE - 1) / LINE_SIZE;
        let mut cursor_line = starting_line;

        let mut available_lines = 0;
        for index in (0..starting_line).rev() {
            if !self.is_occupied_at(index) {
                available_lines += 1;
                if index == 0 && available_lines >= lines_required {
                    let limit = index * LINE_SIZE;
                    let cursor = cursor_line * LINE_SIZE;
                    return Some((cursor, limit));
                }
            } else {
                if available_lines > lines_required {
                    let limit = (index + 2) * LINE_SIZE;
                    let cursor = cursor_line * LINE_SIZE;
                    return Some((cursor, limit));
                }
                available_lines = 0;
                cursor_line = index;
            }
        }

        None
    }

    /// Reset all mark flags to unmarked.
    pub fn reset(&mut self) {
        unsafe {
            for i in 0..LINE_COUNT {
                *self.lines.add(i) = 0;
            }
        }
    }

    unsafe fn as_line_mark(&mut self, line: usize) -> &mut u8 {
        &mut *self.lines.add(line)
    }

    fn is_occupied_at(&self, line_index: usize) -> bool {
        let marked = unsafe { *self.lines.add(line_index) };
        marked != 0
    }
}

impl From<BlockError> for AllocError {
    fn from(error: BlockError) -> AllocError {
        match error {
            BlockError::BadRequest => AllocError::BadRequest,
            BlockError::OOM => AllocError::OOM,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_find_next_hole() {
        // A set of marked lines with a couple holes.
        // The first hole should be seen as conservatively marked.
        // The second hole should be the one selected.
        let block = Block::new(BLOCK_SIZE).unwrap();
        let mut meta = BlockMeta::new(block.as_ptr());

        // #: marked, o: found hole
        // 0123456789A
        // ### # oooo#
        //       L   C
        meta.mark_line(0);
        meta.mark_line(1);
        meta.mark_line(2);
        meta.mark_line(4);
        meta.mark_line(10);

        let expect = Some((10 * LINE_SIZE, 6 * LINE_SIZE));
        let got = meta.find_next_available_hole(10 * LINE_SIZE, LINE_SIZE);

        assert!(got == expect);
    }

    #[test]
    fn test_find_next_hole_at_line_zero() {
        // Should find the hole starting at the beginning of the block
        let block = Block::new(BLOCK_SIZE).unwrap();
        let mut meta = BlockMeta::new(block.as_ptr());

        // #: marked, o: found hole
        // 0123456789A
        // ooo###
        // L  C
        meta.mark_line(3);
        meta.mark_line(4);
        meta.mark_line(5);

        let expect = Some((3 * LINE_SIZE, 0));
        let got = meta.find_next_available_hole(3 * LINE_SIZE, LINE_SIZE);

        assert!(got == expect);
    }

    #[test]
    fn test_find_next_hole_at_block_end() {
        // The first half of the block is marked.
        // The second half of the block should be identified as a hole.
        let block = Block::new(BLOCK_SIZE).unwrap();
        let mut meta = BlockMeta::new(block.as_ptr());

        // #: marked, o: found hole
        // 0....H....C
        // o...o######
        // L    C
        let halfway = LINE_COUNT / 2;
        for i in halfway..LINE_COUNT {
            meta.mark_line(i);
        }

        // because halfway line should be conservatively marked
        let expect = Some((halfway * LINE_SIZE, 0));
        let got = meta.find_next_available_hole(BLOCK_CAPACITY, LINE_SIZE);

        assert!(got == expect);
    }

    #[test]
    fn test_find_hole_all_conservatively_marked() {
        // Every other line is marked.
        // No hole should be found.
        let block = Block::new(BLOCK_SIZE).unwrap();
        let mut meta = BlockMeta::new(block.as_ptr());

        // #: marked, o: found hole
        // 0.........C
        // # # # # # #
        for i in (0..LINE_COUNT).step_by(2) {
            meta.mark_line(i);
        }

        let got = meta.find_next_available_hole(BLOCK_CAPACITY, LINE_SIZE);
        assert!(got == None);
    }

    #[test]
    fn test_find_entire_block() {
        // No marked lines. Entire block is available.
        let block = Block::new(BLOCK_SIZE).unwrap();
        let meta = BlockMeta::new(block.as_ptr());

        // #: marked, o: found hole
        // 0.........C
        // ooooooooooo
        // L         C
        let expect = Some((BLOCK_CAPACITY, 0));
        let got = meta.find_next_available_hole(BLOCK_CAPACITY, LINE_SIZE);

        assert!(got == expect);
    }
}
