//! Tests for the boot memory map (`src/subs/mem/mod.rs`).

use runix_tests::subs::boot::multiboot2::MemoryArea;
use runix_tests::subs::mem::{BootMemoryMap, FRAME_SIZE, MAX_MEMORY_REGIONS, ReservedRange};
use runix_tests::testutil::{memory_areas, TYPE_AVAILABLE, TYPE_UNUSABLE};

/// Shorthand to build a map from (base, len, type) tuples.
pub fn map_from(entries: &[(u64, u64, u32)]) -> Option<BootMemoryMap> {
    let areas: &[MemoryArea] = memory_areas(entries);
    BootMemoryMap::from_multiboot(areas)
}

#[test]
fn only_available_areas_survive() {
    let map = map_from(&[
        (0x0010_0000, 0x0100_0000, TYPE_AVAILABLE),
        (0xfffc_0000, 0x0004_0000, TYPE_UNUSABLE),
        (0x0020_0000, 0x0020_0000, TYPE_AVAILABLE),
    ])
    .unwrap();

    let regions = map.regions();
    assert_eq!(regions.len(), 2);
    assert_eq!(regions[0].start(), 0x0010_0000);
    assert_eq!(regions[1].start(), 0x0020_0000);
}

#[test]
fn regions_are_frame_aligned() {
    // 0xff123 aligns up to 0x100000; end 0x112BDF aligns down to 0x112000.
    let map = map_from(&[(0x000f_f123, 0x0001_3abc, TYPE_AVAILABLE)]).unwrap();
    let r = map.regions()[0];
    assert_eq!(r.start(), 0x0010_0000);
    assert_eq!(r.end(), 0x0011_2000);
    assert_eq!(r.start() % FRAME_SIZE, 0);
    assert_eq!((r.end() - 1) % FRAME_SIZE, (FRAME_SIZE - 1));
}

#[test]
fn align_to_frames_is_idempotent_and_trims_edges() {
    let map = map_from(&[(0x000f_f123, 0x0001_3abc, TYPE_AVAILABLE)]).unwrap();
    let aligned = map.regions()[0].align_to_frames();
    assert_eq!(aligned.start(), 0x0010_0000);
    assert_eq!(aligned.end(), 0x0011_2000);
}

#[test]
fn zero_start_area_is_dropped() {
    // The loader commonly reports low memory at 0x0; the map drops it.
    let map = map_from(&[(0x0000_0000, 0x0010_0000, TYPE_AVAILABLE)]).unwrap();
    assert!(map.regions().is_empty());
}

#[test]
fn empty_len_area_is_dropped() {
    let map = map_from(&[(0x0010_0000, 0x0000_0000, TYPE_AVAILABLE)]).unwrap();
    assert!(map.regions().is_empty());
}

#[test]
fn capacity_is_loud_failure() {
    // More usable areas than MAX_MEMORY_REGIONS must return None, not
    // silently truncate the boot map.
    let entries: Vec<(u64, u64, u32)> = (0..(MAX_MEMORY_REGIONS + 1))
        .map(|i| (0x0010_0000 + (i as u64) * 0x0010_0000, 0x0010_0000, TYPE_AVAILABLE))
        .collect();
    assert!(map_from(&entries).is_none());
}

#[test]
fn empty_reserved_range_is_a_noop() {
    let mut map = map_from(&[(0x0010_0000, 0x0010_0000, TYPE_AVAILABLE)]).unwrap();
    assert!(map.reserve(ReservedRange::new(0x0012_0000, 0x0012_0000)));
    assert_eq!(map.regions().len(), 1);
    assert_eq!(map.regions()[0].size(), 0x0010_0000);
}

#[test]
fn reserved_range_normalizes_reversed_bounds() {
    // ReservedRange::new lower/upper normalizes by min/max, so a "reversed"
    // call still reserves the covered span.
    let mut map = map_from(&[(0x0000_1000, 0x0000_3000, TYPE_AVAILABLE)]).unwrap();
    assert!(map.reserve(ReservedRange::new(0x0000_3000, 0x0000_1000)));
    let regions = map.regions();

    // [0x1000, 0x4000) has [0x1000, 0x3000) removed; the [0x3000, 0x4000) tail
    // survives at the aligned boundary.
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].start(), 0x0000_3000);
    assert_eq!(regions[0].end(), 0x0000_4000);
}

#[test]
fn reserve_splits_region_in_two() {
    // Region 0x10000..0x30000; reserve the middle page 0x18000..0x19000.
    let mut map = map_from(&[(0x0001_0000, 0x0002_0000, TYPE_AVAILABLE)]).unwrap();
    assert!(map.reserve(ReservedRange::new(0x0001_8000, 0x0001_9000)));

    let regions = map.regions();
    assert_eq!(regions.len(), 2);
    assert_eq!(regions[0].start(), 0x0001_0000);
    assert_eq!(regions[0].end(), 0x0001_8000);
    assert_eq!(regions[1].start(), 0x0001_9000);
    assert_eq!(regions[1].end(), 0x0003_0000);
}

#[test]
fn reserve_takes_head_and_tail_pieces() {
    let mut map = map_from(&[(0x0001_0000, 0x0002_0000, TYPE_AVAILABLE)]).unwrap();

    assert!(map.reserve(ReservedRange::new(0x0001_0000, 0x0001_2000)));
    let regions = map.regions();
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].start(), 0x0001_2000);

    assert!(map.reserve(ReservedRange::new(0x0001_e000, 0x0002_0000)));
    assert_eq!(map.regions()[0].end(), 0x0001_e000);
}

#[test]
fn reserve_hole_covering_whole_region_drops_it() {
    let mut map = map_from(&[(0x0001_0000, 0x0002_0000, TYPE_AVAILABLE)]).unwrap();
    assert!(map.reserve(ReservedRange::new(0x0000_0000, 0x0004_0000)));
    assert!(map.regions().is_empty());
}

#[test]
fn reserve_outside_does_not_touch_regions() {
    let mut map = map_from(&[(0x0001_0000, 0x0002_0000, TYPE_AVAILABLE)]).unwrap();
    assert!(map.reserve(ReservedRange::new(0x0010_0000, 0x0011_0000)));
    assert_eq!(map.regions().len(), 1);
    assert_eq!(map.regions()[0].end(), 0x0003_0000); // region is [0x10000, 0x30000)
}

#[test]
fn reserve_across_multiple_regions() {
    // Two disjoint regions; the reserved hole spans both.
    let mut map = map_from(&[
        (0x0001_0000, 0x0001_0000, TYPE_AVAILABLE),
        (0x0002_0000, 0x0001_0000, TYPE_AVAILABLE),
    ])
    .unwrap();
    assert!(map.reserve(ReservedRange::new(0x0001_8000, 0x0002_8000)));

    let regions = map.regions();
    assert_eq!(regions.len(), 2);
    assert_eq!(regions[0].end(), 0x0001_8000);
    assert_eq!(regions[1].start(), 0x0002_8000);
}

#[test]
fn reserve_overflow_is_loud() {
    // The map is at capacity; punching a hole through the middle of one
    // region would split it into two (33 slots > 32 cap) → reserve fails.
    let entries: Vec<(u64, u64, u32)> = (0..MAX_MEMORY_REGIONS)
        .map(|i| (0x0010_0000 + (i as u64) * 0x0010_0000, 0x0010_0000, TYPE_AVAILABLE))
        .collect();
    let mut map = map_from(&entries).unwrap();

    // Region 1 spans [0x20_0000, 0x30_0000): a hole strictly inside it.
    let r = ReservedRange::new(0x0028_0000, 0x0029_0000);
    assert!(!map.reserve(r));
}
