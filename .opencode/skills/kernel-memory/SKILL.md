---
name: kernel-memory
description: Boot memory map and physical frame allocator invariants for runix. Use when touching reserved ranges, the boot memory map, frame allocation, or page alignment.
---

## Layout

- `src/subs/mem/mod.rs` — `BootMemoryMap`, `MemoryRegion`, `ReservedRange`, constants.
- `src/subs/mem/reusables.rs` — `FrameAllocator` struct, tracking limits, shared `FRAME_BITMAP`.
- `src/subs/mem/operators.rs` — the `impl FrameAllocator` (`new`/`alloc`/`free`/counters).

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

- Bitmap-backed physical frame allocator over the reserved-filtered map (one bit per
  4 KiB frame, up to `MAX_TRACKED_FRAMES` frames / 256 MiB). Construction is fallible:
  `FrameAllocator::new` returns `None` when the boot map exceeds tracking capacity.
- Keeps its own copy of the region list plus per-region frame offsets, leaving the
  original `BootMemoryMap` inspectable.
- `alloc()` returns the first free frame (first-fit) and marks it used.
- `free(frame)` validates alignment, region ownership, and that the frame is currently
  allocated; clears the bit and updates counters. Double frees and foreign frames
  return `false`.
- Tracks `total_frames`, `used_frames`, and `free_frames()` counters.
- Returns `None` when all regions are exhausted.

## Before paging

There is no virtual memory yet — addresses are physical. The VGA buffer is memory-mapped at `0xb8000`; any new MMIO must be reserved the same way as the ranges above.
