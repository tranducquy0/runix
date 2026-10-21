//! Core kernel subsystems: `arch` (CPU descriptor-table setup), `boot`
//! (Multiboot2 handoff/parsing), `mem` (boot memory map + frame allocator),
//! and `handlers` (panic and future interrupt dispatch).

pub mod arch;
pub mod boot;
pub mod handlers;
pub mod mem;
