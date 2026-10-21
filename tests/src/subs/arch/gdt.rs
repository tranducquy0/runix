// Thin bridge: paste the kernel's GDT module so tests compile the same
// descriptor logic the kernel ships. The `cfg(target_arch = "x86")` parts
// (global asm stub, extern declaration, init) stay off on this host build.
include!("../../../../src/subs/arch/gdt.rs");
