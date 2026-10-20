//! Multiboot2 support, split by data flow:
//!
//! - [`header`] — the 40-byte image header (plus magic/checksum constants)
//!   that *GRUB reads from our image* in `.multiboot2`.
//! - [`boot_info`] — the boot-info block *we read from GRUB's ebx handoff*,
//!   walked at runtime to find the memory-map tag.
//! - [`memory_area`] — the raw 24-byte memory-area entry, the shared
//!   vocabulary between the parser and the `mem` subsystem.
//!
//! The re-exports below keep the pre-split public path
//! `subs::boot::multiboot2::X` working for `main.rs`, `subs/mem`, and the
//! host-test shims.

pub mod boot_info;
pub mod header;
pub mod memory_area;

pub use boot_info::{BOOTLOADER_MAGIC, BootInformation};
pub use header::{
    CHECKSUM, Multiboot2Header, MULTIBOOT2_ARCHITECTURE_I386, MULTIBOOT2_HEADER_LENGTH,
    MULTIBOOT2_HEADER_MAGIC,
};
pub use memory_area::MemoryArea;
// The kernel image never names `MemoryMapTag` directly (it flows through
// `BootInformation::memory_map_tag()`), so keep the old public path alive
// without an unused-import warning in the binary.
#[allow(unused_imports)]
pub use boot_info::MemoryMapTag;
