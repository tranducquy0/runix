// Thin bridge: paste the kernel allocator definitions (constants, shared
// bitmap, `FrameAllocator` struct) into this module so
// `crate::subs::mem::reusables::FrameAllocator` matches the kernel tree.
include!("../../../../src/subs/mem/reusables.rs");
