//! Multiboot2 boot-information parser.
//!
//! GRUB hands the kernel a physical address (via `ebx`) pointing at a
//! boot-info block laid out as:
//!
//! ```text
//! [BootInformationHeader] [tag] [tag] ... [end tag]
//! ```
//!
//! Every tag starts with a common [`TagHeader`] (type + size) and tags are
//! 8-byte aligned back-to-back. This module walks that block to find the
//! memory-map tag (type 6) and exposes its entries zero-copy; ownership and
//! mutation are left to higher layers (`BootMemoryMap`).
//!
//! Trust model: the block comes from the bootloader, so parsing stays
//! bounded by the advertised `total_size` and degrades to "no memory map"
//! on malformed input instead of hanging the boot.

use core::{marker::PhantomData, mem, slice};

/// Value GRUB must place in `eax` for the boot info in `ebx` to be valid.
pub const BOOTLOADER_MAGIC: u32 = 0x36d7_6289;
/// Tag type of the memory-map (e820-style) tag.
const MEMORY_MAP_TAG_TYPE: u32 = 6;
/// Area `type` value marking usable ("available") physical memory.
const MEMORY_AREA_TYPE_AVAILABLE: u32 = 1;

/// Fixed-size prefix of the boot-info block, read once to learn its extent.
///
/// Layout (8 bytes): `total_size` covers this header plus every aligned tag
/// that follows; `_reserved` must be zero but is never trusted.
#[repr(C)]
struct BootInformationHeader {
    // Total size covers the fixed header plus every aligned tag that follows.
    total_size: u32,
    _reserved: u32,
}

/// Common prologue shared by every tag in the boot-info block.
///
/// `size` counts the whole tag (header + payload) up to — but not including
/// — the padding that aligns the *next* tag to 8 bytes. Type 0 with size 8
/// is the canonical end-of-block marker.
#[repr(C)]
struct TagHeader {
    // Every Multiboot2 boot-info tag starts with these two fields.
    typ: u32,
    size: u32,
}

/// Fixed 16-byte header of the memory-map tag, preceding the area entries.
///
/// `entry_size` is the stride GRUB uses per entry (24 in practice; the spec
/// allows it to grow in future revisions) and `entry_version` is currently
/// always 0.
#[repr(C)]
struct MemoryMapTagHeader {
    typ: u32,
    size: u32,
    entry_size: u32,
    entry_version: u32,
}

/// One raw physical memory area as reported by the firmware via GRUB.
///
/// Matches the Multiboot2 memory-map entry layout exactly (24 bytes):
/// `base_addr`/`length` are byte-granular, `typ` says whether the range is
/// usable (1) or reserved/ACPI/other (everything else).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemoryArea {
    base_addr: u64,
    length: u64,
    typ: u32,
    _reserved: u32,
}

impl MemoryArea {
    // First byte of the area.
    pub fn start_address(&self) -> u64 {
        self.base_addr
    }

    // Byte length of the area (not a frame count).
    pub fn size(&self) -> u64 {
        self.length
    }

    // True only for the "available" e820 type; everything else is reserved.
    pub fn is_available(&self) -> bool {
        self.typ == MEMORY_AREA_TYPE_AVAILABLE
    }
}

/// Borrowed view of one bootloader-provided boot-info block.
///
/// Produced by [`BootInformation::load`]; remembers only the block's
/// `[start, end)` extent so tag walking can be bounded by it. The block
/// itself stays owned (and possibly reused) by the bootloader.
pub struct BootInformation {
    start: usize,
    end: usize,
}

impl BootInformation {
    // Read the fixed header and remember the block extent. Unsafe: `addr` must point at valid Multiboot2 boot info.
    pub unsafe fn load(addr: usize) -> Self {
        // This pointer is bootloader-owned. Read only the fixed-size prefix here
        // so later parsing can stay bounded by the advertised total size.
        let header = unsafe { &*(addr as *const BootInformationHeader) };
        Self {
            start: addr,
            end: addr + header.total_size as usize,
        }
    }

    // Physical address the block starts at (used to reserve it from the frame allocator).
    pub fn start_address(&self) -> usize {
        self.start
    }

    // One-past-the-end address of the block, i.e. `start + total_size`.
    pub fn end_address(&self) -> usize {
        self.end
    }

    // Walk the tag list and return the memory-map tag, or None if absent/malformed.
    pub fn memory_map_tag(&self) -> Option<MemoryMapTag<'_>> {
        // After the boot-info header, tags are stored back-to-back with 8-byte
        // padding between them. We stop at the end tag or when the block ends.
        let mut offset = self.start + mem::size_of::<BootInformationHeader>();

        while offset < self.end {
            let header = unsafe { &*(offset as *const TagHeader) };

            if header.typ == MEMORY_MAP_TAG_TYPE {
                let tag = unsafe { &*(offset as *const MemoryMapTagHeader) };
                let entries_offset = offset + mem::size_of::<MemoryMapTagHeader>();
                let entries_size = header.size as usize - mem::size_of::<MemoryMapTagHeader>();
                let entry_count = entries_size / tag.entry_size as usize;

                return Some(MemoryMapTag {
                    // The parser stays zero-copy here. Higher layers can copy
                    // into owned kernel structures when they need mutation.
                    entries: entries_offset as *const MemoryArea,
                    entry_count,
                    _marker: PhantomData,
                });
            }

            // Type 0 with size 8 is the canonical Multiboot2 end tag.
            if header.typ == 0 && header.size == 8 {
                break;
            }

            // A zero-size tag cannot occur in a valid boot-info block and
            // would make this walk spin forever (offset += 0). Guard against
            // a malformed block: stop and report no memory map instead of
            // hanging the whole boot.
            if header.size == 0 {
                break;
            }

            offset += align_up(header.size as usize, 8);
        }

        None
    }
}

/// Zero-copy handle to the memory-map tag's entries inside the boot-info block.
///
/// Holds a raw pointer into bootloader-owned memory plus a `PhantomData`
/// tying the entries' lifetime to the borrowed [`BootInformation`]. Entry
/// `i` logically lives at `entries + i * 24`; the caller (higher layers)
/// copies into owned structures before mutating anything.
pub struct MemoryMapTag<'a> {
    entries: *const MemoryArea,
    entry_count: usize,
    _marker: PhantomData<&'a MemoryArea>,
}

impl<'a> MemoryMapTag<'a> {
    // Reinterpret the tag payload as a slice of memory areas.
    pub fn memory_areas(&self) -> &'a [MemoryArea] {
        // Entries are packed contiguously after the memory-map tag header.
        unsafe { slice::from_raw_parts(self.entries, self.entry_count) }
    }
}

// Round `value` up to the next multiple of `align` (a power of two).
const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}
