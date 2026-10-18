//! Tests for the frame allocator (`src/subs/mem/allocator.rs`).
//!
//! Everything lives in ONE test function on purpose: the allocator keeps its
//! bitmap in a shared static (single boot-time instance inside the kernel),
//! so parallel test threads would race on it. Add cases inside this single
//! test, sequentially.

use runix_tests::subs::boot::multiboot2::MemoryArea;
use runix_tests::subs::mem::allocator::FrameAllocator;
use runix_tests::subs::mem::{BootMemoryMap, FRAME_SIZE};
use runix_tests::testutil::{memory_areas, TYPE_AVAILABLE};

/// usize conversion helper so the frame arithmetic stays u64-clean and then
/// narrows once for the host target.
fn addr(v: u64) -> usize {
    v as usize
}

fn allocator_from(entries: &[(u64, u64, u32)]) -> Option<FrameAllocator> {
    let areas: &[MemoryArea] = memory_areas(entries);
    BootMemoryMap::from_multiboot(areas)
        .as_mut()
        .and_then(|m| FrameAllocator::new(m))
}

#[test]
fn frame_allocator_invariants() {
    // --- construction and counters ---
    let mut alloc = allocator_from(&[
        // 8 frames: [0x100000, 0x108000)
        (0x0010_0000, 0x0000_8000, TYPE_AVAILABLE),
        // 4 frames: [0x200000, 0x204000)
        (0x0020_0000, 0x0000_4000, TYPE_AVAILABLE),
    ])
    .expect("construction must succeed");

    assert_eq!(alloc.total_frames(), 12);
    assert_eq!(alloc.used_frames(), 0);
    assert_eq!(alloc.free_frames(), 12);

    // --- sequential allocation walks the first region ---
    for i in 0..8u64 {
        let f = alloc.alloc().expect("first region has 8 frames");
        assert_eq!(f, addr(0x0010_0000 + i * (FRAME_SIZE as u64)));
    }
    assert_eq!(alloc.used_frames(), 8);

    // --- crossing into the next region, not assuming flat RAM ---
    for i in 0..4u64 {
        let f = alloc.alloc().expect("second region has 4 frames");
        assert_eq!(f, addr(0x0020_0000 + i * (FRAME_SIZE as u64)));
    }
    assert_eq!(alloc.used_frames(), 12);

    // --- exhausted ---
    assert!(alloc.alloc().is_none());
    assert_eq!(alloc.free_frames(), 0);

    // --- free + realloc returns the same frame ---
    assert!(alloc.free(addr(0x0010_0000)));
    assert_eq!(alloc.used_frames(), 11);
    assert_eq!(alloc.free_frames(), 1);
    assert_eq!(alloc.alloc(), Some(addr(0x0010_0000)));
    assert_eq!(alloc.used_frames(), 12);
    assert!(alloc.free(addr(0x0020_0000)));
    assert_eq!(alloc.alloc(), Some(addr(0x0020_0000)));

    // --- double free is rejected ---
    assert!(alloc.free(addr(0x0010_0000))); // first free of the live frame: ok
    assert!(!alloc.free(addr(0x0010_0000))); // freeing the free frame again

    // --- other invalid frees are rejected ---
    assert!(!alloc.free(addr(0x0010_0004))); // misaligned
    assert!(!alloc.free(addr(0x0030_0000))); // outside any region
    assert!(!alloc.free(addr(0x0011_0000))); // the hole between the regions

    // counters: exactly one accepted free among the batch above
    assert_eq!(alloc.used_frames(), 11);
    assert_eq!(alloc.free_frames(), 1);

    // --- contains semantics ---
    assert!(alloc.contains(addr(0x0010_0000)));
    assert!(alloc.contains(addr(0x0020_3000)));
    assert!(!alloc.contains(addr(0x0010_0004))); // misaligned
    assert!(!alloc.contains(addr(0x0011_0000))); // hole between the regions
    assert!(!alloc.contains(addr(0x0000_9000))); // reserved low memory
}

#[test]
fn capacity_exceeded_is_loud() {
    // One chunk large enough to exceed the tracked-frame cap makes
    // construction fail instead of silently truncating memory.
    let entries: [(u64, u64, u32); 1] = [(0x0010_0000, 0x1000_1000, TYPE_AVAILABLE)];
    assert!(allocator_from(&entries).is_none());
}
