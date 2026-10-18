# Boot Flow

This document describes how `runix` is entered by GRUB and how early control reaches Rust code.

## Current Boot Protocol

`runix` currently boots through Multiboot2 via GRUB.

The GRUB menu entry is defined in [iso/boot/grub/grub.cfg](../iso/boot/grub/grub.cfg):

```cfg
multiboot2 /boot/rkrnl
boot
```

The kernel image is an `elf32-i386` executable linked by [scripts/linker.ld](../scripts/linker.ld).

## Kernel Entry Contract

The linker entry point is `_start`.

At `_start`, the bootloader is expected to provide:

- `eax = 0x36d76289`
  This is the Multiboot2 bootloader magic.
- `ebx = physical address of the Multiboot2 boot information structure`

The kernel currently assumes:

- 32-bit x86 execution
- the CPU is already in protected mode
- paging is either disabled or set up in a way that still makes the loaded kernel image directly usable
- VGA text memory is available at `0xb8000`
- interrupts are not required during early boot

## Multiboot2 Header

The Multiboot2 header is emitted from [src/main.rs](../src/main.rs) into the `.multiboot2` section.

The linker keeps that section near the start of the image:

```ld
.multiboot2 : ALIGN(8) {
    KEEP(*(.multiboot2))
}
```

The current header requests one boot information tag:

- memory map tag (`type = 6`)

This means GRUB is asked to include the physical memory map in the boot information passed through `ebx`.

## Early Assembly Stub

The first instructions executed live in [src/asm/boot.asm](../src/asm/boot.asm), included with `global_asm!` from [src/main.rs](../src/main.rs).

The stub does three things:

1. reserves a temporary 32 KiB boot stack in `.bss` (grew from 4 KiB: the
   debug-build `kernel_main` frame plus `println!` formatting machinery
   overflows 4 KiB and corrupts/overruns the boot stack before any panic
   handler could report it)
2. sets `esp` to the top of that stack
3. forwards `eax` and `ebx` to Rust using the C calling convention

Equivalent shape:

```asm
_start:
    mov esp, offset boot_stack_top
    push ebx
    push eax
    call kernel_main
```

The push order matters. On 32-bit x86 C ABI, arguments are pushed right-to-left, so:

- `push ebx` passes the second Rust parameter
- `push eax` passes the first Rust parameter

That matches:

```rust
extern "C" fn kernel_main(multiboot_magic: u32, multiboot_info_addr: u32) -> !;
```

If `kernel_main` ever returns, the stub halts in an infinite loop.

## Rust Boot Handoff

`kernel_main` is the first Rust function in the boot path.

Its current responsibilities are:

1. clear the VGA screen (so GRUB's menu text doesn't bleed into kernel output),
   then print an early banner through the VGA text writer
2. verify `multiboot_magic == 0x36d76289`
3. treat `multiboot_info_addr` as a Multiboot2 boot information pointer
4. locate the memory map tag
5. copy the `Available` areas into a fixed-capacity `BootMemoryMap`, frame-aligned to 4 KiB
6. reserve the kernel image, the Multiboot2 boot information, and the boot stack
7. print the remaining usable regions
8. smoke-test the `FrameAllocator` by allocating a few frames, freeing one,
   and reallocating to prove the bitmap tracks reuse

If the magic is wrong, or the memory map tag is missing (or the map/allocator
cannot be built), the kernel prints an error and loops forever.

## Boot Information Parsing

The local Multiboot2 parser lives in [src/subs/boot/multiboot2.rs](../src/subs/boot/multiboot2.rs).

It exists because the external `multiboot2` crate path used earlier pulled in `alloc`, which is not available in this kernel build.

The parser currently supports:

- boot information header parsing
- generic tag walking with 8-byte alignment
- memory map tag discovery
- iteration over memory areas

The parser currently treats memory area type `1` as usable RAM.

## Current Memory Map Usage

The memory map now feeds real allocation policy:

- only memory areas reported as `Available` are kept
- regions are aligned to 4 KiB frame boundaries
- kernel image, Multiboot2 boot information, and boot stack ranges are reserved
  (splitting regions where needed) so the allocator never hands them out
- the filtered regions seed the `FrameAllocator`, which hands out physical frames

See [architecture.md](architecture.md) for the memory subsystem in more detail.

## Limitations

This boot path is intentionally minimal and still has several limitations:

- no IDT/GDT setup is documented here yet
- no paging setup
- no serial console fallback
- no handoff from temporary boot stack to a long-lived kernel stack
- no validation beyond the bootloader magic and presence of the memory map tag

## Practical Checks

Useful commands while working on boot:

```bash
grub-file --is-x86-multiboot2 target/i686-runix-target/debug/runix && echo ok
readelf -x .multiboot2 target/i686-runix-target/debug/runix
readelf -l target/i686-runix-target/debug/runix
```

## Next Steps

The next useful extensions to this document are:

- describe the exact Multiboot2 memory map format consumed by the kernel
- define the transition from boot-time stack/setup into a real kernel initialization phase
- document future paging and higher-half plans once they exist
