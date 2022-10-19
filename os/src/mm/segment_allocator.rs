use alloc::vec::Vec;
use crate::mm::{frame_alloc, FrameTracker, PhysPageNum};
use crate::mm::frame_allocator::FRAME_ALLOCATOR;

struct Segment {
    size: usize,
    frames: Vec<FrameTracker>,
}


struct SegmentTracker {
    seg_index: usize,
}


struct SegmentAllocator{
    segments: Vec<Segment>,
    bitmap: Vec<bool>,
}

impl SegmentAllocator {
    fn new() -> Self {
        Self {
            segments : Vec::new(),
            bitmap : Vec::new(),
        }
    }

    fn alloc(&mut self, alloc_size: usize ) -> Option<usize> {

    //     first fit
        let mut idx = 0;
        let mut result = -1;
        for segment in self.segments {
            if segment.size > alloc_size {
                result = idx;
                break;
            }else {
                idx += 1;
            }
        }

        if result == -1 { None } else { Some(result) }
    }

    fn dealloc(&mut self, seg_index: usize) {

    }

}

impl SegmentAllocator {
    pub fn init(&mut self) {
        let mut remaining = FRAME_ALLOCATOR.exclusive_access().remaining();
        let size_pick = [1,2,4,8,16];
        let mut count = 0;
        while remaining > 0 {
            let size = if size_pick[count % 5] < remaining { size_pick[count % 5] } else { remaining };
            let mut frames = Vec::new();
            for _i in 0..size {
                frames.push(frame_alloc().unwrap())
            }
            self.segments.push(Segment {
                size,
                frames,
            });
            self.bitmap.push(false);

            count += 1;
            remaining -= size;
        }
    }
}
