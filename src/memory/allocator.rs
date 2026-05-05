use super::{BootMemoryMap, FRAME_SIZE, MAX_MEMORY_REGIONS, MemoryRegion};

pub struct FrameAllocator {
    // The allocator keeps its own cursor state so the original boot memory map
    // can still be inspected or reused later during initialization.
    regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    region_count: usize,
    current_region: usize,
    next_frame: usize,
}

impl FrameAllocator {
    pub fn new(memory_map: &BootMemoryMap) -> Self {
        let mut regions = [MemoryRegion::empty(); MAX_MEMORY_REGIONS];
        let source = memory_map.regions();

        let mut i = 0;
        while i < source.len() {
            regions[i] = source[i];
            i += 1;
        }

        Self {
            regions,
            region_count: source.len(),
            current_region: 0,
            // Zero means "not started in the current region yet".
            next_frame: 0,
        }
    }

    pub fn alloc(&mut self) -> Option<usize> {
        // The allocator walks one region at a time. When a region is exhausted,
        // it moves forward instead of assuming all RAM is one flat range.
        while self.current_region < self.region_count {
            let region = self.regions[self.current_region];
            let next = if self.next_frame == 0 {
                region.start()
            } else {
                self.next_frame
            };

            let end = region.end();
            if next >= end {
                // This region is exhausted, so advance to the next usable one.
                self.current_region += 1;
                self.next_frame = 0;
                continue;
            }

            // `BootMemoryMap` already aligned region edges to page boundaries, so
            // we only need to step one frame forward after each allocation.
            self.next_frame = next + FRAME_SIZE;
            return Some(next);
        }

        None
    }
}
