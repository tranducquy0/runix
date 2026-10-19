// Boot-time physical memory map.
//
// NOTE: this file is `include!`d by the host-test shim, so it must use plain
// `//` comments at module level (inner `//!` docs are illegal in expansions).
//
// Before any frame allocator exists, the kernel needs a fixed-capacity
// description of usable RAM that it can carve reserved ranges out of.
// This module provides that:
//
// - `BootMemoryMap` — fixed array of `MemoryRegion`s, seeded from the
//   bootloader's Multiboot2 memory areas (available types only,
//   frame-aligned) and then split around `ReservedRange`s (kernel image,
//   boot stack, boot-info block).
// - `MemoryRegion` — a half-open `[start, end)` byte range that is
//   always 4 KiB frame-aligned once it lives in the map.
// - `ReservedRange` — an occupied span to punch out of the map.
//
// Design rules:
// - No allocation: everything is `[T; MAX_MEMORY_REGIONS]` on the stack.
// - Failure is loud: overflowing the 32-slot capacity returns `None`/
//   `false` so boot stops with a message instead of silently losing RAM.
// - Reservation alignment only ever *shrinks* usable regions, so a frame
//   partially covered by a reserved range is never handed out.

pub mod reusables;
pub mod operators;

//(re-exported so callers keep a short `mem::FrameAllocator` path)
pub use self::reusables::FrameAllocator;

use crate::subs::boot::multiboot2::MemoryArea;

/// Maximum usable regions the boot map (and the allocator) can track.
pub const MAX_MEMORY_REGIONS: usize = 32;
/// Frame size the whole memory subsystem works in units of (x86 4 KiB pages).
pub const FRAME_SIZE: usize = 4096;

/// Shared zero-value for a region slot: `[0, 0)`, i.e. empty.
const EMPTY_REGION: MemoryRegion = MemoryRegion { start: 0, end: 0 };

/// Half-open byte span `[start, end)` that must not be handed out.
///
/// Constructed from linker symbols (`__kernel_start`/`__kernel_end`, the
/// boot stack) and the bootloader's boot-info block; passed to
/// [`BootMemoryMap::reserve`]. Bounds are normalized on construction so
/// callers may pass `(high, low)` without breaking reservation logic.
#[derive(Clone, Copy)]
pub struct ReservedRange {
    start: usize,
    end: usize,
}

impl ReservedRange {
    // Build a range, normalizing reversed bounds with min/max.
    pub fn new(start: usize, end: usize) -> Self {
        // Normalize once so callers do not need to worry about ordering.
        Self {
            start: start.min(end),
            end: start.max(end),
        }
    }

    // True when the span covers no bytes (start == end); a no-op to reserve.
    fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// One usable, frame-aligned physical region `[start, end)`.
///
/// Stored by value in both the boot map and the frame allocator. Once in a
/// [`BootMemoryMap`], `start` and `end` are guaranteed multiples of
/// [`FRAME_SIZE`] (see [`MemoryRegion::align_to_frames`]).
#[derive(Clone, Copy)]
pub struct MemoryRegion {
    start: usize,
    end: usize,
}

impl MemoryRegion {
    // Empty region constant, used to pre-fill fixed arrays.
    pub const fn empty() -> Self {
        EMPTY_REGION
    }

    // First byte of the region.
    pub fn start(&self) -> usize {
        self.start
    }

    // One-past-the-last byte (exclusive end).
    pub fn end(&self) -> usize {
        self.end
    }

    // Byte size of the region (saturating; never negative).
    pub fn size(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    // Shrink to fully contained frames: start up, end down (never grows).
    pub fn align_to_frames(self) -> Self {
        // Regions are treated as [start, end), so aligning `start` up and `end`
        // down safely drops any partial frame at either edge.
        Self {
            start: align_up(self.start, FRAME_SIZE),
            end: align_down(self.end, FRAME_SIZE),
        }
    }

    // Build a region, normalizing reversed bounds; used while splitting.
    fn new(start: usize, end: usize) -> Self {
        Self {
            start: start.min(end),
            end: start.max(end),
        }
    }

    // True when the region covers no whole bytes/frames.
    fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// Fixed-capacity set of usable regions describing all RAM the kernel may use.
///
/// Built once per boot by [`BootMemoryMap::from_multiboot`], then mutated
/// only by [`BootMemoryMap::reserve`] (which punches out occupied spans,
/// splitting regions as needed). The frame allocator later copies the final
/// region list, so this map stays inspectable for diagnostics.
pub struct BootMemoryMap {
    // No allocator exists during early boot, so this is a fixed-capacity array
    // instead of a growable collection.
    regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    len: usize,
}

impl BootMemoryMap {
    // Convert Multiboot2 areas into frame-aligned usable regions, skipping non-available ones.
    pub fn from_multiboot(areas: &[MemoryArea]) -> Option<Self> {
        let mut map = Self::new();

        for area in areas {
            // GRUB reports many memory types, but only `Available` can seed the
            // frame allocator.
            if !area.is_available() {
                continue;
            }

            let start = match usize::try_from(area.start_address()) {
                Ok(start) => start,
                Err(_) => continue,
            };
            let size = match usize::try_from(area.size()) {
                Ok(size) => size,
                Err(_) => continue,
            };

            let end = start.saturating_add(size);
            if start == 0 || end <= start {
                continue;
            }

            // The bootloader can describe usable memory with byte precision, but
            // the frame allocator only hands out whole 4 KiB pages. Aligning the
            // regions here keeps all later code working with the same contract.
            let region = MemoryRegion::new(start, end).align_to_frames();
            if region.is_empty() {
                continue;
            }
            if !map.push(region) {
                // Failing loudly is better than silently truncating the boot map.
                return None;
            }
        }

        Some(map)
    }

    // Punch a reserved span out of the map, splitting regions around it; false on capacity overflow.
    pub fn reserve(&mut self, reserved: ReservedRange) -> bool {
        if reserved.is_empty() {
            return true;
        }

        let mut next = [EMPTY_REGION; MAX_MEMORY_REGIONS];
        let mut next_len = 0;

        // Reserving memory can split one usable region into two smaller regions.
        // We rebuild the region list so later allocator code only sees safe ranges.
        for region in self.regions() {
            // No overlap means the region survives unchanged.
            if reserved.end <= region.start || reserved.start >= region.end {
                if !push_region(&mut next, &mut next_len, *region) {
                    return false;
                }
                continue;
            }

            // Keep the portion before the reserved range, if there is one.
            if region.start < reserved.start {
                let left = MemoryRegion::new(region.start, reserved.start.min(region.end))
                    .align_to_frames();
                if !left.is_empty() && !push_region(&mut next, &mut next_len, left) {
                    return false;
                }
            }

            // Keep the portion after the reserved range, if there is one.
            if reserved.end < region.end {
                let right = MemoryRegion::new(reserved.end.max(region.start), region.end)
                    .align_to_frames();
                if !right.is_empty() && !push_region(&mut next, &mut next_len, right) {
                    return false;
                }
            }
        }

        self.regions = next;
        self.len = next_len;
        true
    }

    // Borrow the live usable regions (`&regions[..len]`).
    pub fn regions(&self) -> &[MemoryRegion] {
        &self.regions[..self.len]
    }

    // Empty map with zeroed region slots.
    fn new() -> Self {
        Self {
            regions: [EMPTY_REGION; MAX_MEMORY_REGIONS],
            len: 0,
        }
    }

    // Append one region; false if empty or the fixed capacity is exhausted.
    fn push(&mut self, region: MemoryRegion) -> bool {
        push_region(&mut self.regions, &mut self.len, region)
    }
}

// Round `addr` up to the next multiple of `align` (a power of two).
const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

// Round `addr` down to the previous multiple of `align` (a power of two).
const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

// Shared fixed-array append used by both construction and reserve(); rejects empty slots.
fn push_region(
    regions: &mut [MemoryRegion; MAX_MEMORY_REGIONS],
    len: &mut usize,
    region: MemoryRegion,
) -> bool {
    // Empty regions are ignored, and overflow tells the caller it needs either
    // a larger fixed-capacity map or a different early-boot strategy.
    if region.is_empty() || *len >= MAX_MEMORY_REGIONS {
        return false;
    }

    regions[*len] = region;
    *len += 1;
    true
}
