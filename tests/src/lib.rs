//! Host-side shim crate that re-exposes the kernel's logic modules for tests.
//!
//! `src/subs.rs` mirrors the kernel's `subs` tree with thin files that pull in
//! the real kernel sources (via `#[path]`/`include!`), so the tests exercise
//! the *same code* the kernel ships. Hardware-coupled code (VGA writer, boot
//! asm, panic handler) stays out and is covered by the live QEMU checks
//! (`make vm agent && make vm screen`).

pub mod subs;

pub mod testutil {
    use crate::subs::boot::multiboot2::{BootInformation, MemoryArea};

    // Mirror the private layout in multiboot2.rs to fabricate boot data.
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct RawMemoryArea {
        base_addr: u64,
        length: u64,
        typ: u32,
        reserved: u32,
    }

    pub const TYPE_AVAILABLE: u32 = 1;
    pub const TYPE_UNUSABLE: u32 = 4;

    /// Fabricate a `&[MemoryArea]` with the exact `repr(C)` layout the parser
    /// expects, without depending on private constructor access.
    pub fn memory_areas(entries: &[(u64, u64, u32)]) -> &[MemoryArea] {
        assert_layouts();
        let raw: Vec<RawMemoryArea> = entries
            .iter()
            .map(|&(base, len, typ)| RawMemoryArea {
                base_addr: base,
                length: len,
                typ,
                reserved: 0,
            })
            .collect();
        // Leaked on purpose: each fabricated map must outlive its test.
        let leaked: &'static [RawMemoryArea] = raw.leak();
        // SAFETY: identical layout asserted above, and the fabricated areas
        // are only ever read (val := true) by the kernel's parser code paths.
        unsafe { std::slice::from_raw_parts(leaked.as_ptr() as *const MemoryArea, leaked.len()) }
    }

    fn assert_layouts() {
        assert_eq!(
            std::mem::size_of::<RawMemoryArea>(),
            std::mem::size_of::<MemoryArea>()
        );
    }

    /// Build a minimal Multiboot2 boot-info block carrying a memory-map tag
    /// with the given areas (base, len, type) per entry.
    pub fn multiboot2_with_memory_map(area_entries: &[(u64, u64, u32)]) -> Vec<u8> {
        // The memory-map tag: TagHeader (8) + entry_size + entry_version (8)
        // + entry_count * 24 bytes; plus the terminating end tag.
        let tag_size = 16 + area_entries.len() * 24;
        let total = 8 + tag_size + 8;
        let mut buf = Vec::with_capacity(total);

        buf.extend_from_slice(&(total as u32).to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes()); // reserved

        buf.extend_from_slice(&6u32.to_le_bytes()); // memory map tag type
        buf.extend_from_slice(&(tag_size as u32).to_le_bytes());
        buf.extend_from_slice(&24u32.to_le_bytes()); // entry_size
        buf.extend_from_slice(&0u32.to_le_bytes()); // entry_version
        for &(base, len, typ) in area_entries {
            buf.extend_from_slice(&base.to_le_bytes());
            buf.extend_from_slice(&len.to_le_bytes());
            buf.extend_from_slice(&typ.to_le_bytes());
            buf.extend_from_slice(&0u32.to_le_bytes()); // entry reserved
        }

        buf.extend_from_slice(&0u32.to_le_bytes()); // end tag type
        buf.extend_from_slice(&8u32.to_le_bytes()); // end tag size
        buf
    }

    /// Load an encoded block into a `BootInformation`.
    pub fn boot_info_from_block(block: &[u8]) -> BootInformation {
        unsafe { BootInformation::load(block.as_ptr() as usize) }
    }
}
