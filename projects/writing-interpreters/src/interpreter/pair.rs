use std::cell::Cell;

use super::{error::SourcePos, safeptr::TaggedCellPtr};

/// A Pair of pointers, like a Cons cell of old
#[derive(Clone)]
pub struct Pair {
    pub first: TaggedCellPtr,
    pub second: TaggedCellPtr,
    // Possible source code positions of the first and second values
    pub first_pos: Cell<Option<SourcePos>>,
    pub second_pos: Cell<Option<SourcePos>>,
}
