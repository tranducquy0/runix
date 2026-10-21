// Architecture-specific CPU setup: descriptor tables today, a TSS once
// interrupts or ring 3 land.

pub mod gdt;
