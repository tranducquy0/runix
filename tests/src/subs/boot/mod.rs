// Thin bridge to the kernel's Multiboot2 parser; resolves to the real file
// two levels up and to the repo root: runix/src/subs/boot/multiboot2.rs.
#[path = "../../../../src/subs/boot/multiboot2.rs"]
pub mod multiboot2;
