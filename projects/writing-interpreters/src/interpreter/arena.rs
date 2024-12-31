use crate::memory::StickyImmixHeap;

/// Allocation header for an Arena-allocated value
pub struct ArenaHeader {}

/// A non-garbage-collected pool of memory blocks for interned values.
/// These values are not dropped on Arena deallocation.
/// Values must be "atomic", that is, not composed of other object
/// pointers that need to be traced.
pub struct Arena {
    heap: StickyImmixHeap<ArenaHeader>,
}
