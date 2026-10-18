//! Tests for the Multiboot2 boot-info parser (`src/subs/boot/multiboot2.rs`).

use runix_tests::subs::boot::multiboot2::BOOTLOADER_MAGIC;
use runix_tests::testutil::{boot_info_from_block, multiboot2_with_memory_map, TYPE_AVAILABLE};

// A well-formed block with two available areas parses entry-for-entry.
#[test]
fn parses_memory_map_entries() {
    let block = multiboot2_with_memory_map(&[
        (0x0000_0000, 0x0009_fc00, TYPE_AVAILABLE),
        (0x0010_0000, 0x0100_0000, TYPE_AVAILABLE),
    ]);
    let info = boot_info_from_block(&block);
    let tag = info.memory_map_tag().expect("memmap tag must be found");

    let areas = tag.memory_areas();
    assert_eq!(areas.len(), 2);
    assert_eq!(areas[0].start_address(), 0x0000_0000);
    assert_eq!(areas[0].size(), 0x0009_fc00);
    assert!(areas[0].is_available());
    assert_eq!(areas[1].start_address(), 0x0010_0000);
    assert_eq!(areas[1].size(), 0x0100_0000);
    assert!(areas[1].is_available());
}

// Non-available areas pass through the parser and report is_available() == false.
#[test]
fn unavailable_areas_are_reported_as_such() {
    let block = multiboot2_with_memory_map(&[(0xc000_0000, 0x0040_0000, 4)]);
    let info = boot_info_from_block(&block);
    let tag = info.memory_map_tag().unwrap();
    assert_eq!(tag.memory_areas().len(), 1);
    assert!(!tag.memory_areas()[0].is_available());
}

// Header + end tag only: the walker returns None instead of guessing.
#[test]
fn missing_memory_map_tag_returns_none() {
    // Header + end tag only; no tags in between.
    let mut block = Vec::new();
    block.extend_from_slice(&16u32.to_ne_bytes()); // total_size
    block.extend_from_slice(&0u32.to_ne_bytes()); // reserved
    block.extend_from_slice(&0u32.to_ne_bytes()); // end tag type
    block.extend_from_slice(&8u32.to_ne_bytes()); // end tag size
    let info = boot_info_from_block(&block);
    assert!(info.memory_map_tag().is_none());
}

// Unknown tag types are skipped with correct 8-byte alignment, not fatal.
#[test]
fn tag_walk_skips_unknown_tags_before_memmap() {
    // Put a bogus tag (type 99) before the memory map; the walker must skip
    // it (12-byte payload walks as 16 under 8-byte alignment) and still find
    // the memory map tag afterwards.
    let mut block = Vec::new();

    let total_size = 8 + 16 + 40 + 8; // header, unknown tag (padded), memmap, end
    block.extend_from_slice(&(total_size as u32).to_le_bytes());
    block.extend_from_slice(&0u32.to_le_bytes()); // reserved

    // unknown tag: header(8) + 4-byte payload; multiboot2 tags round up to
    // 8-byte multiples, so the block carries 4 bytes of padding after it
    block.extend_from_slice(&99u32.to_le_bytes());
    block.extend_from_slice(&12u32.to_le_bytes());
    block.extend_from_slice(&[0u8; 4]); // tag payload
    block.extend_from_slice(&[0u8; 4]); // padding to the tag's 16-byte stride


    // memory map tag with one area
    block.extend_from_slice(&6u32.to_le_bytes());
    block.extend_from_slice(&40u32.to_le_bytes());
    block.extend_from_slice(&24u32.to_le_bytes()); // entry_size
    block.extend_from_slice(&0u32.to_le_bytes()); // entry_version
    block.extend_from_slice(&(0x0010_0000u64).to_le_bytes());
    block.extend_from_slice(&(0x0020_0000u64).to_le_bytes());
    block.extend_from_slice(&TYPE_AVAILABLE.to_le_bytes());
    block.extend_from_slice(&0u32.to_le_bytes());

    block.extend_from_slice(&0u32.to_le_bytes()); // end tag type
    block.extend_from_slice(&8u32.to_le_bytes()); // end tag size

    let info = boot_info_from_block(&block);
    let tag = info.memory_map_tag().expect("walker must pass unknown tag");
    assert_eq!(tag.memory_areas().len(), 1);
    assert_eq!(tag.memory_areas()[0].start_address(), 0x0010_0000);
    assert_eq!(tag.memory_areas()[0].size(), 0x0020_0000);
}

// The end tag bounds the walk: tags placed after it are never observed.
#[test]
fn end_tag_is_honored() {
    // A memory-map tag placed after the end tag must NOT be picked up.
    let mut block = Vec::new();

    block.extend_from_slice(&((8u32 + 8)).to_ne_bytes()); // total_size: header + end
    block.extend_from_slice(&0u32.to_ne_bytes());

    block.extend_from_slice(&0u32.to_ne_bytes()); // end tag
    block.extend_from_slice(&8u32.to_ne_bytes());

    block.extend_from_slice(&6u32.to_ne_bytes());
    block.extend_from_slice(&40u32.to_ne_bytes());
    block.extend_from_slice(&24u32.to_ne_bytes());
    block.extend_from_slice(&0u32.to_ne_bytes());
    block.extend_from_slice(&(0x0010_0000u64).to_le_bytes());
    block.extend_from_slice(&(0x0010_0000u64).to_le_bytes());
    block.extend_from_slice(&TYPE_AVAILABLE.to_le_bytes());
    block.extend_from_slice(&0u32.to_le_bytes());

    let info = boot_info_from_block(&block);
    assert!(info.memory_map_tag().is_none());
}

// Pin the magic constant so a typo here can't desync from the Multiboot2 spec.
#[test]
fn bootloader_magic_constant_matches_spec() {
    assert_eq!(BOOTLOADER_MAGIC, 0x36d7_6289);
}
