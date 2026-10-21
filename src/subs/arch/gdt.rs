//! Global descriptor table: the kernel stops borrowing GRUB's segments.
//!
//! GRUB enters us in 32-bit protected mode on its own flat GDT; this module
//! installs the kernel's equivalents (null + flat ring-0 code + flat ring-0
//! data) and reloads every segment register in one uninterrupted assembly
//! step ([`init`], wired to `asm/gdt.asm`), so nothing after boot depends on
//! descriptor state GRUB happens to leave behind. Selector `0x18` stays
//! reserved for a TSS once interrupts or ring 3 arrive.

#[cfg(target_arch = "x86")]
use core::arch::global_asm;

/// Selector (GDT index << 3) of the kernel's own flat code segment.
/// Must stay in sync with the hard-coded selectors in `asm/gdt.asm`.
pub const KERNEL_CODE_SELECTOR: u16 = 0x08;

/// Selector (GDT index << 3) of the kernel's own flat data segment.
/// Must stay in sync with the hard-coded selectors in `asm/gdt.asm`.
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;

/// One 8-byte segment descriptor, bit-packed exactly the way the CPU decodes
/// it: bits 0-15 limit[0..16], 16-31 base[0..16], 32-39 base[16..24],
/// 40-47 access byte, 48-51 limit[16..20], 52-55 flags, 56-63 base[24..32].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SegmentDescriptor(pub u64);

impl SegmentDescriptor {
    /// CPU convention: a fully zero descriptor must exist as entry 0 and
    /// must never be selected (loading it faults).
    pub const NULL: Self = Self(0);

    /// Pack the raw descriptor fields. `flags` carries only the 4 flag bits
    /// (granularity, default operand size, long mode, AVL); `limit` and
    /// `base` are split across their low/high fields per the layout above.
    pub const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        Self(
            (limit & 0xFFFF) as u64
                | ((base & 0xFFFF) as u64) << 16
                | (((base >> 16) & 0xFF) as u64) << 32
                | (access as u64) << 40
                | (((limit >> 16) & 0xF) as u64) << 48
                | ((flags & 0xF) as u64) << 52
                | ((base >> 24) as u64) << 56,
        )
    }
}

// Access byte for a present ring-0 code segment (executable, readable).
const ACCESS_FLAT_RING0_CODE: u8 = 0x9A;
// Access byte for a present ring-0 data segment (readable, writable).
const ACCESS_FLAT_RING0_DATA: u8 = 0x92;
// Flags nibble: 4 KiB granularity + 32-bit default operand size.
const FLAGS_4K_GRANULARITY_32BIT: u8 = 0xC;
// Limit paired with 4 KiB granularity so one flat entry spans 4 GiB.
const FLAT_LIMIT: u32 = 0xFFFFF;

// The table is 8-byte aligned so its first entry lands on a descriptor
// boundary, and `#[used]`-adjacent normal statics place it inside the kernel
// image ranges the boot-reservation code already protects.
#[repr(align(8))]
struct GlobalDescriptorTable {
    entries: [SegmentDescriptor; 3],
}

#[used]
static GDT: GlobalDescriptorTable = GlobalDescriptorTable {
    entries: [
        SegmentDescriptor::NULL,
        SegmentDescriptor::new(0, FLAT_LIMIT, ACCESS_FLAT_RING0_CODE, FLAGS_4K_GRANULARITY_32BIT),
        SegmentDescriptor::new(0, FLAT_LIMIT, ACCESS_FLAT_RING0_DATA, FLAGS_4K_GRANULARITY_32BIT),
    ],
};

#[cfg(target_arch = "x86")]
global_asm!(include_str!("../../asm/gdt.asm"));

// The lgdt operand in 32-bit mode: a 2-byte limit followed by the 4-byte
// linear base of the table, matching the CPU's 48-bit GDTR layout.
#[cfg(target_arch = "x86")]
#[repr(C, packed)]
struct DescriptorTablePointer {
    limit: u16,
    base: u32,
}

#[cfg(target_arch = "x86")]
unsafe extern "C" {
    // Defined in `asm/gdt.asm`: lgdt + reload data segments + far-return CS.
    fn gdt_load_flush(gdt_pointer_addr: u32);
}

/// Install the kernel's own flat ring-0 GDT and reload every segment
/// register on top of it in one uninterrupted asm sequence.
///
/// Runs before anything else in `kernel_main` so no later code can depend on
/// GRUB's descriptors. There is no failure path: a bad pointer or selector
/// here triple-faults the CPU instead of returning an error.
#[cfg(target_arch = "x86")]
pub fn init() {
    let pointer = DescriptorTablePointer {
        limit: (GDT.entries.len() * core::mem::size_of::<SegmentDescriptor>() - 1) as u16,
        base: &raw const GDT as u32,
    };

    // SAFETY: the pointer describes the statically sized table above, which
    // lives inside the reservation already carved out for the kernel image;
    // the stub swaps ds/es/fs/gs/ss and CS back-to-back with no intermediate
    // Rust code executing.
    unsafe { gdt_load_flush(&raw const pointer as u32) };
}
