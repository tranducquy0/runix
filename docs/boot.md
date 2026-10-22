# Boot flow

How GRUB enters `runix`, and how control reaches Rust.

## Protocol

`runix` boots through Multiboot2 via GRUB. The menu entry is defined in
[iso/boot/grub/grub.cfg](../iso/boot/grub/grub.cfg):

```cfg
menuentry "R-Unix Alpha 0.0.1" {
    set gfxpayload=text
    multiboot2 /boot/rkrnl
    boot
}
```

The kernel image is an `elf32-i386` executable linked by
[scripts/linker.ld](../scripts/linker.ld) with `ENTRY(_start)`, loaded at 1 MiB.

## Entry contract

The linker entry point is `_start`. At that point the bootloader provides:

- `eax = 0x36d76289` — the Multiboot2 bootloader magic
- `ebx` — physical address of the Multiboot2 boot information structure

The kernel assumes:

- 32-bit x86 execution, CPU already in protected mode
- paging either disabled or set up so the loaded kernel image is directly usable
- VGA text memory available at `0xb8000`
- interrupts not required during early boot

## Multiboot2 header

The header is emitted from [src/main.rs](../src/main.rs) into the `.multiboot2`
section as raw bytes (magic `0xE85250D6`, architecture i386, length 40, checksum),
followed by an information-request tag and the required end tag. Raw bytes keep the
on-disk layout exactly what the spec demands, independent of Rust layout rules.

The linker keeps the section near the start of the image, inside the first 32 KiB GRUB
scans:

```ld
.multiboot2 : ALIGN(8) {
    KEEP(*(.multiboot2))
}
```

The single information request is for the memory map (`type = 6`), so GRUB includes the
physical memory map in the boot information passed through `ebx`.

## Assembly stub

The first executed instructions live in [src/asm/boot.asm](../src/asm/boot.asm),
included with `global_asm!` from [src/main.rs](../src/main.rs). The stub:

1. reserves a temporary 32 KiB boot stack in `.bss` — grown from 4 KiB, which
   overflowed once `kernel_main` gained larger locals plus the `println!` formatting
   machinery (frames corrupted adjacent data before the panic handler could report it)
2. sets `esp` to the top of that stack
3. forwards `eax` and `ebx` to Rust using the C calling convention

Shape:

```asm
_start:
    mov esp, offset boot_stack_top
    push ebx        # second argument
    push eax        # first argument
    call kernel_main
```

Push order matters: on 32-bit x86 C ABI arguments are pushed right-to-left, which
matches:

```rust
extern "C" fn kernel_main(multiboot_magic: u32, multiboot_info_addr: u32) -> !;
```

If `kernel_main` ever returns, the stub halts in a `hlt` loop.

## Rust handoff

`kernel_main` (first Rust function in the boot path):

1. clears the VGA screen (otherwise GRUB's menu text bleeds into kernel output) and
   prints an early banner
2. verifies `multiboot_magic == 0x36d76289`
3. treats `multiboot_info_addr` as the boot information pointer
4. locates the memory map tag
5. copies the `Available` areas into a fixed-capacity `BootMemoryMap` (`MAX_MEMORY_REGIONS = 32`), frame-aligned to 4 KiB
6. reserves the kernel image, the Multiboot2 boot information, and the boot stack
   (splitting regions as needed)
7. prints the remaining usable regions
8. builds the `FrameAllocator` and smoke-tests it: allocate a few frames, free one,
   reallocate to prove the bitmap tracks reuse

Every failure path — wrong magic, missing memory map tag, map that does not fit,
reservation that splits too often, allocator over tracking capacity — prints an error
and loops forever.

## Boot-information parsing

The parser lives in [src/subs/boot/multiboot2.rs](../src/subs/boot/multiboot2.rs). It
exists in-tree because the external `multiboot2` crate pulls in `alloc`, which this
kernel does not have.

It supports: boot information header parsing, generic tag walking with 8-byte
alignment, memory map tag discovery, and iteration over memory areas (type `1` = usable
RAM). The memory map then feeds allocation policy in `src/subs/mem/` — see
[architecture.md](architecture.md).

## Limitations

- no paging setup
- interrupts disabled (IDT installed for exceptions only; no scheduling)
- no serial console fallback — VGA text only
- no handoff from the boot stack to a long-lived kernel stack
- validation beyond the bootloader magic and the memory map tag's presence

## Practical checks

```bash
grub-file --is-x86-multiboot2 target/i686-runix-target/debug/runix && echo ok
readelf -x .multiboot2 target/i686-runix-target/debug/runix
readelf -l target/i686-runix-target/debug/runix
```
