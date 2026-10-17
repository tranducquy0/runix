pub mod allocator;

use crate::subs::boot::multiboot2::MemoryArea;

pub const MAX_MEMORY_REGIONS: usize = 32;
pub const FRAME_SIZE: usize = 4096;

const EMPTY_REGION: MemoryRegion = MemoryRegion { start: 0, end: 0 };

#[derive(Clone, Copy)]
pub struct ReservedRange {
    start: usize,
    end: usize,
}

impl ReservedRange {
    pub fn new(start: usize, end: usize) -> Self {
        // Normalize once so callers do not need to worry about ordering.
        Self {
            start: start.min(end),
            end: start.max(end),
        }
    }

    fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

#[derive(Clone, Copy)]
pub struct MemoryRegion {
    start: usize,
    end: usize,
}

impl MemoryRegion {
    pub const fn empty() -> Self {
        EMPTY_REGION
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn size(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn align_to_frames(self) -> Self {
        // Regions are treated as [start, end), so aligning `start` up and `end`
        // down safely drops any partial frame at either edge.
        Self {
            start: align_up(self.start, FRAME_SIZE),
            end: align_down(self.end, FRAME_SIZE),
        }
    }

    fn new(start: usize, end: usize) -> Self {
        Self {
            start: start.min(end),
            end: start.max(end),
        }
    }

    fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

pub struct BootMemoryMap {
    // No allocator exists during early boot, so this is a fixed-capacity array
    // instead of a growable collection.
    regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    len: usize,
}

impl BootMemoryMap {
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
            if !map.push(region) {
                // Failing loudly is better than silently truncating the boot map.
                return None;
            }
        }

        Some(map)
    }

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

    pub fn regions(&self) -> &[MemoryRegion] {
        &self.regions[..self.len]
    }

    fn new() -> Self {
        Self {
            regions: [EMPTY_REGION; MAX_MEMORY_REGIONS],
            len: 0,
        }
    }

    fn push(&mut self, region: MemoryRegion) -> bool {
        push_region(&mut self.regions, &mut self.len, region)
    }
}

const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

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
