// Thin bridge: paste the kernel's TSS module here so its layout can be
// exercised on the host, keeping crate::subs::arch paths identical to the
// kernel.
include!("../../../../src/subs/arch/tss.rs");
