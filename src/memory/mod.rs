pub mod allocator;

use crate::boot::multiboot2::MemoryArea;

pub const MAX_MEMORY_REGIONS: usize = 32;

const EMPTY_REGION: MemoryRegion = MemoryRegion { start: 0, end: 0 };

#[derive(Clone, Copy)]
pub struct ReservedRange {
    start: usize,
    end: usize,
}

impl ReservedRange {
    pub fn new(start: usize, end: usize) -> Self {
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
    regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    len: usize,
}

impl BootMemoryMap {
    pub fn from_multiboot(areas: &[MemoryArea]) -> Option<Self> {
        let mut map = Self::new();

        for area in areas {
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

            if !map.push(MemoryRegion::new(start, end)) {
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
            if reserved.end <= region.start || reserved.start >= region.end {
                if !push_region(&mut next, &mut next_len, *region) {
                    return false;
                }
                continue;
            }

            if region.start < reserved.start {
                let left = MemoryRegion::new(region.start, reserved.start.min(region.end));
                if !left.is_empty() && !push_region(&mut next, &mut next_len, left) {
                    return false;
                }
            }

            if reserved.end < region.end {
                let right = MemoryRegion::new(reserved.end.max(region.start), region.end);
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

fn push_region(
    regions: &mut [MemoryRegion; MAX_MEMORY_REGIONS],
    len: &mut usize,
    region: MemoryRegion,
) -> bool {
    if region.is_empty() || *len >= MAX_MEMORY_REGIONS {
        return false;
    }

    regions[*len] = region;
    *len += 1;
    true
}
