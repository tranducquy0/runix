// Thin bridge: paste the kernel's arch/mod.rs here so its `pub mod gdt;`
// resolves against tests/src/subs/arch/gdt.rs (the shim below), keeping
// crate::subs::arch paths identical to the kernel.
include!("../../../../src/subs/arch/mod.rs");
