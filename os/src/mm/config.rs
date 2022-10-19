use crate::mm::frame_allocator::{BitMapFrameAllocator, LinkedListAllocator, StackFrameAllocator};

// Choose A specific Frame Allocator
#[cfg(feature = "stackframe_allocator")]
pub type FrameAllocatorImpl = StackFrameAllocator;
#[cfg(feature = "bitmap_allocator")]
pub type FrameAllocatorImpl = BitMapFrameAllocator;
#[cfg(feature = "linkedlist_allocator")]
pub type FrameAllocatorImpl = LinkedListAllocator;


// Choose A specific Memory Allocator for MapArea
#[cfg(feature = "frame_allocator")]
pub const MEMORY_ALLOCATOR:MemoryAllocator = MemoryAllocator::Frame;
#[cfg(feature = "segment_allocator")]
pub const MEMORY_ALLOCATOR:MemoryAllocator = MemoryAllocator::Segment;




// If we intend to test the frame allocator
pub const TEST_FRAME: bool = false;

// If we intend to test the segment allocator
pub const TEST_SEGMENT: bool = false;

// Specific allocating model for map area
pub enum MemoryAllocator {
    Segment,
    Frame,
}