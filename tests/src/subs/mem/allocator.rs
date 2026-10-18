// Thin bridge: paste the kernel allocator itself into this module so
// `crate::subs::mem::allocator::FrameAllocator` matches the kernel tree.
include!("../../../../src/subs/mem/allocator.rs");
