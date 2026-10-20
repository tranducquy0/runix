//! The raw physical-memory-area entry shared by the boot-info walker and
//! the `mem` subsystem (which copies entries into an owned `BootMemoryMap`).

/// Area `type` value marking usable ("available") physical memory.
pub const MEMORY_AREA_TYPE_AVAILABLE: u32 = 1;

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
