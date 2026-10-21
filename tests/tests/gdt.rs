//! Tests for the flat ring-0 GDT encoding (`src/subs/arch/gdt.rs`).
//!
//! The selector swaps themselves are hardware behavior (lgdt, far return)
//! and stay covered by the live QEMU boot check instead.

use runix_tests::subs::arch::gdt::{
    KERNEL_CODE_SELECTOR, KERNEL_DATA_SELECTOR, SegmentDescriptor,
};

// Entry 0 must stay all-zero by CPU convention: loading it faults.
#[test]
fn null_descriptor_is_all_zero() {
    assert_eq!(SegmentDescriptor::NULL, SegmentDescriptor(0));
}

// Flat 4 GiB ring-0 code segment: every field lands in the byte the CPU
// decodes it from — limit 0xFFFF split + nibble 0xF, base 0, access 0x9A
// (present, ring 0, executable, readable), flags 0xC (4K gran, 32-bit).
#[test]
fn flat_ring0_code_descriptor_encodes_field_for_field() {
    assert_eq!(
        SegmentDescriptor::new(0, 0xF_FFFF, 0x9A, 0xC),
        SegmentDescriptor(0x00CF_9A00_0000_FFFF)
    );
}

// Same flat shape for the data segment, differing only in access byte
// 0x92 (present, ring 0, readable, writable).
#[test]
fn flat_ring0_data_descriptor_encodes_field_for_field() {
    assert_eq!(
        SegmentDescriptor::new(0, 0xF_FFFF, 0x92, 0xC),
        SegmentDescriptor(0x00CF_9200_0000_FFFF)
    );
}

// The packed limit (low 16 bits + high nibble at bits 48-51), decoded back
// against the 4K granularity flag, must cover exactly the 4 GiB address
// space — this is what makes a "flat" segment actually flat.
#[test]
fn flat_limit_with_4k_granularity_spans_exactly_4gib() {
    for encoded in [
        SegmentDescriptor::new(0, 0xF_FFFF, 0x9A, 0xC),
        SegmentDescriptor::new(0, 0xF_FFFF, 0x92, 0xC),
    ] {
        let limit = (encoded.0 & 0xFFFF) as u32 | (((encoded.0 >> 48) & 0xF) as u32) << 16;
        assert_eq!((u64::from(limit) + 1) << 12, 0x1_0000_0000);
    }
}

// Selector contract of the static table: selector = entry index << 3, so
// null is 0x00, code is entry 1, data is entry 2 — and the hard-coded
// selector constants in the asm stub must keep matching.
#[test]
fn selectors_pin_the_table_layout() {
    assert_eq!(KERNEL_CODE_SELECTOR, 0x08); // entry 1, ring 0
    assert_eq!(KERNEL_DATA_SELECTOR, 0x10); // entry 2, ring 0
}
