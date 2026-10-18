# Tests for runix kernel modules (host-side)

The kernel binary itself is a freestanding i686 image and cannot run its own
`cargo test` (`test = false`, `no_std`, custom target). This crate compiles the
kernel's *logic* modules on the host target and exercises them with normal rust
tests:

- `boot` — the Multiboot2 boot-info walker (`src/subs/boot/multiboot2.rs`)
- `mem` — `BootMemoryMap`, `MemoryRegion`, `ReservedRange`, `FrameAllocator`
  (`src/subs/mem/`)

Hardware-coupled code (VGA writer, panic handler, boot asm) is not compiled
here; live-boot checks happen through QEMU: `make vm agent` then `make vm
screen`.

Run all of it:

```sh
make tests        # host-side logic tests (rust)
make build        # kernel
make iso test     # image-level validation (ELF + Multiboot2)
make vm agent && make vm screen   # boot the real image and read its screen
```

The `.cargo/config.toml` inside this directory overrides the repo root's
`[build] target` so the tests build for the host instead of `i686-runix-target`.
