# Architecture

`runix` is a `no_std`, `no_main` i686 kernel. It boots over Multiboot2 (via GRUB),
takes an early memory map from the bootloader, and starts handing out physical frames.
Everything else is still to come.

## Layout

```
src/
├── main.rs                  kernel entry, Multiboot2 header, boot handoff
├── asm/boot.asm             raw boot stub (_start): stack setup + GRUB handoff
├── subs/                    core subsystems
│   ├── boot/multiboot2.rs   Multiboot2 boot-info parser (header + tag walking)
│   ├── mem/mod.rs           boot memory map, reserved-range filtering
│   ├── mem/allocator.rs     physical frame allocator (bitmap-backed)
│   └── handlers/panic.rs    panic handler
└── drivers/                 hardware drivers
    ├── output/vga.rs        VGA text driver + print!/println! macros
    └── input/               placeholder for future input drivers (e.g. keyboard)
```

Other relevant files:

- [`iso/boot/grub/grub.cfg`](../iso/boot/grub/grub.cfg) — GRUB menu entry
  (`multiboot2 /boot/rkrnl`).
- [`scripts/linker.ld`](../scripts/linker.ld) — linker script; keeps `.multiboot2` near
  the image start.
- [`scripts/kernel_image.sh`](../scripts/kernel_image.sh) — the image validation behind
  `make iso test`.
- [`tests/`](../tests) — host-side test crate that re-exposes the boot parser and
  memory modules via thin `include!` / `#[path]` shims, run with `make tests`.

## Subsystems

### Boot (`src/subs/boot`)

The Multiboot2 parser. It walks the boot-information structure GRUB passes in, finds the
memory-map tag (type 6), and exposes the memory areas zero-copy. It exists in-tree
because the external `multiboot2` crate pulled in `alloc`, which this kernel does not
have. See [boot.md](boot.md) for the entry contract and tag format.

### Memory (`src/subs/mem`)

- `BootMemoryMap` (`mod.rs`) — a fixed-capacity (`MAX_MEMORY_REGIONS = 32`) list of
  usable, frame-aligned regions, built from the Multiboot2 memory map. It copies only
  `Available` areas; if more regions are reported than fit, construction fails loudly
  rather than truncating.
- `ReservedRange` / `BootMemoryMap::reserve` — carves ranges that are already in use
  out of the map, splitting regions as needed so the allocator never sees them.
- `FrameAllocator` (`allocator.rs`) — a bitmap-backed physical frame allocator over the
  reserved-filtered map. One bit per 4 KiB frame, tracking up to `MAX_TRACKED_FRAMES`
  (256 MiB of RAM); `new()` returns `None` instead of truncating when the map exceeds
  that. `alloc()` / `free()` validate alignment and region ownership, so double-free is
  rejected rather than corrupting the bitmap. It never assumes RAM is one flat range.

### Handlers (`src/subs/handlers`)

The `#[panic_handler]`: prints a panic banner and message over VGA, then halts. Future
IDT / interrupt dispatch belongs here.

### Drivers (`src/drivers`)

Split into `output/` and `input/`. `output/vga.rs` is the VGA text-mode writer (80×25 at
`0xb8000`) behind a `spin::Mutex`, and defines the `print!` / `println!` macros
(`#[macro_use] mod drivers;` in `main.rs` keeps them available crate-wide). `input/` is a
placeholder for future drivers such as `keyboard.rs`.

## Boot handoff

GRUB enters the kernel at `_start` with the Multiboot2 magic in `eax` and the boot
information pointer in `ebx`. The assembly stub builds a 32 KiB boot stack, pushes
`ebx` then `eax`, and calls `kernel_main(multiboot_magic, multiboot_info_addr)`.
`kernel_main` validates the magic, builds the memory map, reserves the kernel image,
boot-info block, and boot stack, then feeds the remainder to the `FrameAllocator` and
smoke-tests alloc/free/reuse. The full register/stack contract, header layout, and
failure paths are in [boot.md](boot.md).

## Scope

Beyond the frame allocator smoke test the kernel idles: no scheduler, no userspace, no
filesystem, no interrupts, no paging. See [README.md](../README.md) for overall project
status.
