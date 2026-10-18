//! Core kernel subsystems: `boot` (Multiboot2 handoff/parsing), `mem`
//! (boot memory map + frame allocator), and `handlers` (panic and future
//! interrupt dispatch).

pub mod boot;
pub mod handlers;
pub mod mem;
