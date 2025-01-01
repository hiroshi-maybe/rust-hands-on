use super::{array::Array, safeptr::TaggedCellPtr};

/// A List can contain a mixed sequence of any type of value
pub type List = Array<TaggedCellPtr>;
