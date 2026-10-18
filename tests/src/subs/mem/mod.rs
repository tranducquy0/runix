// Thin bridge: paste the kernel's own mem/mod.rs here so its `pub mod
// allocator;` resolves against tests/src/subs/mem/ (the shim below) and its
// exports (BootMemoryMap, MemoryRegion, ReservedRange, consts) surface under
// the same module path the kernel uses.
include!("../../../../src/subs/mem/mod.rs");
