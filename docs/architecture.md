# Architecture

`runix` is a `no_std`, `no_main` i686 kernel. It boots over Multiboot2 (via GRUB),
takes an early memory map from the bootloader, and starts handing out physical frames.
Everything else is still to come.

## Layout

```
src/
├── main.rs                  kernel entry, Multiboot2 header, boot handoff
├── asm/
│   ├── boot.asm             raw boot stub (_start): stack setup + GRUB handoff
│   ├── gdt.asm              lgdt + segment-register flush, and the TR (ltr) load
│   └── idt.asm              exception stubs + idt_load
├── subs/                    core subsystems
│   ├── boot/multiboot2.rs   Multiboot2 boot-info parser (header + tag walking)
│   ├── arch/gdt.rs          kernel GDT (flat ring-0 code/data, TSS entry)
│   ├── arch/tss.rs          32-bit TSS (privilege-change stack groundwork)
│   ├── mem/mod.rs           boot memory map, reserved-range filtering
│   ├── mem/reusables.rs      frame allocator definitions (bitmap, struct)
│   ├── mem/operators.rs      frame allocator operators (new/alloc/free)
│   ├── handlers/panic.rs    panic handler
│   └── handlers/idt.rs      IDT with exception handlers
└── drivers/                 hardware drivers
    ├── output/vga.rs        VGA text driver + print!/println! macros
    ├── pic.rs               8259 PIC driver, remap + EOI helpers
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

### Arch (`src/subs/arch`)

- `gdt.rs` — installs the kernel's own flat ring-0 GDT (null + code + data) and
  reloads every segment register via the `asm/gdt.asm` stub in one uninterrupted
  sequence, ending the dependency on GRUB's descriptors.
- `tss.rs` — registers an available-32-bit TSS at GDT selector `0x18` (loaded into
  TR during the same `init`). Right now it exists as staged groundwork: `esp0`/
  `ss0` are pointed at the boot stack once interrupts are known to be off, and the
  CPU only consults them on privilege changes that can't happen yet. The stack
  itself is still borrowed from the boot stub (see [boot.md](boot.md) limitations).

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
- `FrameAllocator` (`reusables.rs` + `operators.rs`) — a bitmap-backed physical frame
  allocator over the
  reserved-filtered map. One bit per 4 KiB frame, tracking up to `MAX_TRACKED_FRAMES`
  (256 MiB of RAM); `new()` returns `None` instead of truncating when the map exceeds
  that. `alloc()` / `free()` validate alignment and region ownership, so double-free is
  rejected rather than corrupting the bitmap. It never assumes RAM is one flat range.

### Handlers (`src/subs/handlers`)

- The `#[panic_handler]`: prints a panic banner and message over VGA, then halts.
- `idt.rs` installs the IDT with exception handlers (like panic: print and halt) and
  loads it via `asm/idt.asm`. The 8259 PIC is remapped in `src/drivers/pic.rs` so
  IRQs don't collide with CPU exceptions; interrupts remain disabled for now.
  Double-fault handling (with its own TSS-backed stack) is the staged next step.

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
