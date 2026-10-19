// Frame-allocator definitions: sizing constants, the shared frame bitmap and
// the `FrameAllocator` storage itself. The behavior (construct/alloc/free/
// counters) lives in `operators.rs`.
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

use super::{FRAME_SIZE, MAX_MEMORY_REGIONS, MemoryRegion};

// Upper bound on how much physical memory this allocator can track. The bitmap
// below has one bit per 4 KiB frame up to this limit.
pub const MAX_TRACKED_FRAMES: usize = 256 * 1024 * 1024 / FRAME_SIZE;
// Number of u64 words needed to hold one bit per tracked frame.
const BITMAP_WORDS: usize = (MAX_TRACKED_FRAMES + 63) / 64;

// Stored outside the allocator struct. `static mut` is sound here because the
// frame allocator only exists after GRUB hands control over and nothing runs
// concurrently with it yet.
pub(super) static mut FRAME_BITMAP: [u64; BITMAP_WORDS] = [0u64; BITMAP_WORDS];

/// First-fit frame allocator over a `BootMemoryMap`'s usable regions.
///
/// Allocated addresses are always 4 KiB aligned; `free` accepts only frames
/// the allocator could have produced (aligned, inside a known region, still
/// allocated). Counters (`total`/`used`/`free`) are maintained incrementally
/// for diagnostics.
pub struct FrameAllocator {
    // Usable regions copied out of the `BootMemoryMap`. The allocator keeps its
    // own copy so the original map stays inspectable.
    pub(super) regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    pub(super) region_count: usize,

    // Cumulative frame count before each region. Together with `regions` this
    // maps a global bit index back to a physical frame address.
    pub(super) region_frame_offsets: [usize; MAX_MEMORY_REGIONS],

    // Total number of frames spanned by all regions, and how many are in use.
    pub(super) total_frames: usize,
    pub(super) used_frames: usize,

    // One bit per frame: 1 = allocated, 0 = free. Backed by `FRAME_BITMAP`.
    pub(super) bitmap: &'static mut [u64; BITMAP_WORDS],
}
