// Bitmap-backed physical frame allocator.
//
// NOTE: this file is `include!`d by the host-test shim, so it must use plain
// `//` comments at module level (inner `//!` docs are illegal in expansions).
//
// One bit per 4 KiB frame, stored in a single `static mut` bitmap shared
// by (at most) one live `FrameAllocator` at a time — the allocator only
// exists after GRUB hands control over and nothing runs concurrently yet.
//
// Index space: usable regions from `BootMemoryMap` are concatenated into
// one flat index space (`region_frame_offsets[i]` = first global index of
// region `i`). A global index maps back to a physical address through
// `FrameAllocator::index_to_address`; physical addresses map forward to
// an index by locating their owning region. This keeps the bitmap small
// and region-count-independent while still honoring holes between regions.
//
// Invariants:
// - All usable regions are frame-aligned and non-overlapping (guaranteed
//   by `BootMemoryMap`), so address <-> index mapping is a bijection.
// - A bit is 1 iff the frame has been handed out by `alloc` and not yet
//   returned by `free`; double frees are rejected by checking the bit.
// - Construction fails (`None`) if the map exceeds tracking capacity, so
//   usable RAM is never silently dropped.

use super::{BootMemoryMap, FRAME_SIZE, MAX_MEMORY_REGIONS, MemoryRegion};

// Upper bound on how much physical memory this allocator can track. The bitmap
// below has one bit per 4 KiB frame up to this limit.
pub const MAX_TRACKED_FRAMES: usize = 256 * 1024 * 1024 / FRAME_SIZE;
// Number of u64 words needed to hold one bit per tracked frame.
const BITMAP_WORDS: usize = (MAX_TRACKED_FRAMES + 63) / 64;

// Stored outside the allocator struct. `static mut` is sound here because the
// frame allocator only exists after GRUB hands control over and nothing runs
// concurrently with it yet.
static mut FRAME_BITMAP: [u64; BITMAP_WORDS] = [0u64; BITMAP_WORDS];

/// First-fit frame allocator over a [`BootMemoryMap`]'s usable regions.
///
/// Allocated addresses are always 4 KiB aligned; `free` accepts only frames
/// the allocator could have produced (aligned, inside a known region, still
/// allocated). Counters (`total`/`used`/`free`) are maintained incrementally
/// for diagnostics.
pub struct FrameAllocator {
    // Usable regions copied out of the `BootMemoryMap`. The allocator keeps its
    // own copy so the original map stays inspectable.
    regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    region_count: usize,

    // Cumulative frame count before each region. Together with `regions` this
    // maps a global bit index back to a physical frame address.
    region_frame_offsets: [usize; MAX_MEMORY_REGIONS],

    // Total number of frames spanned by all regions, and how many are in use.
    total_frames: usize,
    used_frames: usize,

    // One bit per frame: 1 = allocated, 0 = free. Backed by `FRAME_BITMAP`.
    bitmap: &'static mut [u64; BITMAP_WORDS],
}

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