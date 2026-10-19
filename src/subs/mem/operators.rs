// Frame-allocator operators: construction, allocation, freeing and the
// diagnostic counters. The storage they operate on lives in `reusables.rs`
// (constants, `FRAME_BITMAP`, `FrameAllocator` struct).
//
// NOTE: this file is `include!`d by the host-test shim, so it must use plain
// `//` comments at module level (inner `//!` docs are illegal in expansions).

use super::reusables::{FrameAllocator, FRAME_BITMAP, MAX_TRACKED_FRAMES};
use super::{BootMemoryMap, FRAME_SIZE, MAX_MEMORY_REGIONS, MemoryRegion};

impl FrameAllocator {
    // Build from a boot map and reset the shared bitmap; None if capacity would be exceeded.
    pub fn new(memory_map: &BootMemoryMap) -> Option<Self> {
        let source = memory_map.regions();
        if source.len() > MAX_MEMORY_REGIONS {
            return None;
        }

        let mut regions = [MemoryRegion::empty(); MAX_MEMORY_REGIONS];
        let mut region_frame_offsets = [0usize; MAX_MEMORY_REGIONS];
        let mut total_frames = 0usize;

        let mut i = 0;
        while i < source.len() {
            regions[i] = source[i];
            region_frame_offsets[i] = total_frames;

            let frames = source[i].size() / FRAME_SIZE;
            total_frames = match total_frames.checked_add(frames) {
                Some(total) => total,
                None => return None,
            };
            if total_frames > MAX_TRACKED_FRAMES {
                return None;
            }

            i += 1;
        }

        // Reset the shared bitmap before handing out a new allocator instance.
        let bitmap = unsafe { &mut *core::ptr::addr_of_mut!(FRAME_BITMAP) };
        for word in bitmap.iter_mut() {
            *word = 0;
        }

        Some(Self {
            regions,
            region_count: source.len(),
            region_frame_offsets,
            total_frames,
            used_frames: 0,
            bitmap,
        })
    }

    // Allocate the lowest free frame (first-fit); None when exhausted.
    pub fn alloc(&mut self) -> Option<usize> {
        // First-fit scan over the global frame index space.
        let mut index = 0;
        while index < self.total_frames {
            let word = index / 64;
            let bit = index % 64;
            let mask = 1u64 << bit;

            if self.bitmap[word] & mask == 0 {
                self.bitmap[word] |= mask;
                self.used_frames += 1;
                return self.index_to_address(index);
            }

            index += 1;
        }

        None
    }

    // Return a frame to the pool; false if misaligned, unknown, or already free.
    pub fn free(&mut self, frame: usize) -> bool {
        // Frames are always handed out aligned, so reject anything else early.
        if frame % FRAME_SIZE != 0 {
            return false;
        }

        // Locate the region that owns this frame.
        let mut i = 0;
        while i < self.region_count {
            let region = self.regions[i];
            if frame >= region.start() && frame < region.end() {
                let local = (frame - region.start()) / FRAME_SIZE;
                let index = self.region_frame_offsets[i] + local;
                let word = index / 64;
                let bit = index % 64;
                let mask = 1u64 << bit;

                // Clearing an already-clear bit would be a double free.
                if self.bitmap[word] & mask == 0 {
                    return false;
                }

                self.bitmap[word] &= !mask;
                self.used_frames = self.used_frames.saturating_sub(1);
                return true;
            }
            i += 1;
        }

        false
    }

    // Total frames tracked across all usable regions.
    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    // Frames currently handed out by alloc().
    pub fn used_frames(&self) -> usize {
        self.used_frames
    }

    // Frames available for allocation (total - used, saturating).
    pub fn free_frames(&self) -> usize {
        self.total_frames.saturating_sub(self.used_frames)
    }

    // True if the frame is aligned and falls inside any usable region (allocated or not).
    #[allow(dead_code)]
    pub fn contains(&self, frame: usize) -> bool {
        if frame % FRAME_SIZE != 0 {
            return false;
        }

        let mut i = 0;
        while i < self.region_count {
            let region = self.regions[i];
            if frame >= region.start() && frame < region.end() {
                return true;
            }
            i += 1;
        }

        false
    }

    // Map a global bit index to its physical address; None if the index falls in no region.
    fn index_to_address(&self, index: usize) -> Option<usize> {
        let mut i = 0;
        while i < self.region_count {
            let start = self.region_frame_offsets[i];
            let frames = self.regions[i].size() / FRAME_SIZE;
            if index >= start && index < start + frames {
                let local = index - start;
                return Some(self.regions[i].start() + local * FRAME_SIZE);
            }
            i += 1;
        }

        None
    }
}
