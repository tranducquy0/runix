use super::{BootMemoryMap, MAX_MEMORY_REGIONS, MemoryRegion};

const FRAME_SIZE: usize = 4096;

pub struct FrameAllocator {
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
            next_frame: 0,
        }
    }

    pub fn alloc(&mut self) -> Option<usize> {
        // The allocator walks one region at a time. When a region is exhausted,
        // it moves forward instead of assuming all RAM is one flat range.
        while self.current_region < self.region_count {
            let region = self.regions[self.current_region];
            let next = if self.next_frame == 0 {
                Self::align_up(region.start())
            } else {
                self.next_frame
            };

            let end = Self::align_down(region.end());
            if next >= end {
                self.current_region += 1;
                self.next_frame = 0;
                continue;
            }

            self.next_frame = next + FRAME_SIZE;
            return Some(next);
        }

        None
    }

    const fn align_up(addr: usize) -> usize {
        (addr + FRAME_SIZE - 1) & !(FRAME_SIZE - 1)
    }

    const fn align_down(addr: usize) -> usize {
        addr & !(FRAME_SIZE - 1)
    }
}
