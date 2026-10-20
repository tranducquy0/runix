// Boot-handoff shim: `multiboot2.rs` in this tree mirrors the kernel
// wrapper (`src/subs/boot/multiboot2.rs`) with per-child `#[path]`
// re-points, so the host tests exercise the real parser sources.
pub mod multiboot2;
