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
│   ├── mem/                 boot memory map, reserved ranges, physical frame allocator
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
  `make test`.

## Subsystems

### Boot (`src/subs/boot`)

The Multiboot2 parser. It walks the boot-information structure GRUB passes in, finds the
memory-map tag (type 6), and exposes the memory areas zero-copy. It exists in-tree
because the external `multiboot2` crate pulled in `alloc`, which this kernel does not
have. See [boot.md](boot.md) for the entry contract and tag format.

### Memory (`src/subs/mem`)

- `BootMemoryMap` — a fixed-capacity (`MAX_MEMORY_REGIONS = 32`) list of usable,
  frame-aligned regions, built from the Multiboot2 memory map. It copies only
  `Available` areas; if more regions are reported than fit, construction fails loudly
  rather than truncating.
- `ReservedRange` / `BootMemoryMap::reserve` — carves ranges that are already in use out
  of the map, splitting regions as needed so the allocator never sees them.
- `FrameAllocator` — a bitmap-backed physical frame allocator over the
  reserved-filtered map. It tracks up to `MAX_TRACKED_FRAMES` frames, supports
  `alloc()` and `free()` with validation, and never assumes RAM is one flat range.

### Handlers (`src/subs/handlers`)

The `#[panic_handler]`: prints a panic banner and message over VGA, then halts. Future
IDT / interrupt dispatch belongs here.

### Drivers (`src/drivers`)

Split into `output/` and `input/`. `output/vga.rs` is the VGA text-mode writer (80×25 at
`0xb8000`) behind a `spin::Mutex`, and defines the `print!` / `println!` macros
(`#[macro_use] mod drivers;` in `main.rs` keeps them available crate-wide). `input/` is a
placeholder for future drivers such as `keyboard.rs`.

## Boot handoff

GRUB enters the kernel at `_start` with `eax = 0x36d76289` (Multiboot2 magic) and `ebx`
pointing at the Multiboot2 boot information. The assembly stub
([`src/asm/boot.asm`](../src/asm/boot.asm)) builds a temporary 4 KiB boot stack in
`.bss`, then pushes `ebx` and `eax` and calls
`kernel_main(multiboot_magic, multiboot_info_addr)`.

`kernel_main` verifies the magic, parses the boot info to find the memory-map tag, copies
only the `Available` areas into a `BootMemoryMap`, and reserves the ranges that are
already in use (kernel image, Multiboot2 info, boot stack) before feeding the remaining
regions to the `FrameAllocator`. The full register/stack contract and limitations are in
[boot.md](boot.md).

## Status

Early, minimal, and honest about it. On boot the kernel is loaded by GRUB, hands control
to Rust, validates the Multiboot2 magic, parses the boot information, builds and filters
the physical memory map, prints the usable regions, and smoke-tests the frame allocator
by allocating a few frames.

After that it idles. There is no scheduler, no userspace, no filesystem, no interrupts,
and no paging yet.
