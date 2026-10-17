---
name: kernel-multiboot2
description: Multiboot2 boot flow for runix — header, GRUB handoff, and boot-info tag parsing. Use when changing the entry path, the Multiboot header, or memory-map parsing.
---

## Two separate Multiboot2 pieces

1. **Header** (what GRUB scans for in the image) — in `src/main.rs`. Constants: `MULTIBOOT2_HEADER_MAGIC = 0xE85250D6`, arch `0` (i386), length `40`, plus a `CHECKSUM` so `magic + arch + len + checksum == 0`. Emitted as a `#[repr(C, align(8))]` `MultibootHeader` of raw bytes to guarantee exact offsets/alignment GRUB expects. Do not let the compiler reorder or pad it.

2. **Boot-info parser** (what GRUB hands us at runtime) — `src/subs/boot/multiboot2.rs`. Validates `BOOTLOADER_MAGIC = 0x36d7_6289` and walks tags.

## Boot flow

```
GRUB -> _start (src/asm/boot.asm) -> kernel_main (src/main.rs)
```

`_start`:
- GRUB enters with Multiboot2 values in `eax` (magic) / `ebx` (boot-info pointer) but no Rust stack frame.
- Sets up `esp = boot_stack_top`, pushes `ebx` then `eax`, `call kernel_main` (C args: magic, info addr).
- On return: `hlt` loop (returning from `kernel_main` is unsupported).

`kernel_main`:
- Checks magic against `multiboot2::BOOTLOADER_MAGIC`; on mismatch prints and halts.
- `BootInformation::load(info_addr)` — reads only the fixed header prefix to bound `total_size`, then the region `[start, end)` is walked for tags.
- `memory_map_tag()` finds tag type `6`. Tags are 8-byte aligned after the header; stop at end tag (`typ == 0 && size == 8`) or block end.

## Gotchas

- Parser is zero-copy: `MemoryArea` entries are read straight from bootloader memory via raw pointers. Copy into owned kernel structures before mutating (that's what `BootMemoryMap::from_multiboot` does).
- `MemoryArea` fields are `u64`; conversion to `usize` can fail on 32-bit — `from_multiboot` skips those areas rather than panicking.
- `align_up(size, 8)` between tags — never assume tags are contiguous without padding.
- `BootInformation::start_address`/`end_address` are reserved as a range in `kernel_main` so the allocator won't reuse GRUB's boot-info block.
