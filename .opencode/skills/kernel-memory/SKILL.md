---
name: kernel-memory
description: Boot memory map and physical frame allocator invariants for runix. Use when touching reserved ranges, the boot memory map, frame allocation, or page alignment.
---

## Layout

- `src/subs/mem/mod.rs` — `BootMemoryMap`, `MemoryRegion`, `ReservedRange`, constants.
- `src/subs/mem/allocator.rs` — `FrameAllocator`.

No heap/allocator crate exists yet. Everything is `no_std`, fixed-capacity, and runs before any paging.

## Invariants (do not break)

- `MAX_MEMORY_REGIONS = 32`. The boot map and the allocator both hold a fixed array of that size. Exceeding it returns `false`/`None` (loud failure), never silently truncates.
- `FRAME_SIZE = 4096`. `BootMemoryMap::from_multiboot` aligns every usable region's `start` up and `end` down to frame boundaries. The allocator relies on this and only steps `next_frame += FRAME_SIZE`.
- Only `Available` (type 1) Multiboot areas seed the map. Everything else is skipped.
- Regions are half-open `[start, end)`. `start == 0` or `end <= start` regions are dropped.

## Reservation order (critical)

`kernel_main` reserves these ranges **before** the allocator ever runs, so it never hands out frames owned by the kernel or bootloader:

1. `__kernel_start` .. `__kernel_end` (linker symbols).
2. GRUB boot info block (`start_address` .. `end_address` from `BootInformation`).
3. `boot_stack` .. `boot_stack_top` (exported from `src/asm/boot.asm`).

`BootMemoryMap::reserve` rebuilds the region list on each call (a reservation can split one region into two). If a split overflows the 32-region cap it returns `false` and `kernel_main` halts. When adding a new reserved range, add it to the `reserved_ranges` array in `kernel_main` (`src/main.rs`), not ad hoc.

## FrameAllocator behavior

- Keeps its own copy of the region list + a cursor (`current_region`, `next_frame`), leaving the original `BootMemoryMap` inspectable.
- `next_frame == 0` means "not started in the current region yet"; it is initialized to the region start on first alloc.
- Walks regions in order; when one is exhausted it advances instead of assuming one flat RAM range. Returns `None` when all regions are used.

## Before paging

There is no virtual memory yet — addresses are physical. The VGA buffer is memory-mapped at `0xb8000`; any new MMIO must be reserved the same way as the ranges above.
