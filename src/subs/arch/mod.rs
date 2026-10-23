// Architecture-specific CPU setup: descriptor tables — the kernel GDT plus a
// registered TSS — today.

pub mod gdt;
pub mod tss;
