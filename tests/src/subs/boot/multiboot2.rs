// Thin bridge to the kernel's Multiboot2 tree: src/subs/boot/multiboot2/.
//
// A single `#[path]` to `multiboot2.rs` no longer works since the wrapper
// gained child modules — children resolve against *this* tree, not the
// real kernel file's location. So this shim mirrors the wrapper shape and
// re-points each child at the real file; `super::` references inside the
// children (e.g. `memory_area` from `boot_info`) then resolve within this
// shimmed subtree, which itself points at the originals.

#[path = "../../../../src/subs/boot/multiboot2/header.rs"]
pub mod header;

#[path = "../../../../src/subs/boot/multiboot2/memory_area.rs"]
pub mod memory_area;

#[path = "../../../../src/subs/boot/multiboot2/boot_info.rs"]
pub mod boot_info;

pub use boot_info::{BOOTLOADER_MAGIC, BootInformation};
pub use header::{
    CHECKSUM, Multiboot2Header, MULTIBOOT2_ARCHITECTURE_I386, MULTIBOOT2_HEADER_LENGTH,
    MULTIBOOT2_HEADER_MAGIC,
};
pub use memory_area::MemoryArea;
// `MemoryMapTag` is only reached through `BootInformation::memory_map_tag()`
// in the tests, but the shim keeps full path parity with the kernel wrapper.
#[allow(unused_imports)]
pub use boot_info::MemoryMapTag;
