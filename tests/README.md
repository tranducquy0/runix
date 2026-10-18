# Host-side tests

The kernel binary can't run `cargo test` itself (`test = false`, `no_std`, freestanding
i686 target). This crate compiles the kernel's *logic* modules for the host and
exercises them with normal rust tests:

- `tests/tests/boot.rs` — the Multiboot2 boot-info walker (`src/subs/boot/multiboot2.rs`)
- `tests/tests/memory_map.rs`, `tests/tests/frame_allocator.rs` — `BootMemoryMap`,
  `MemoryRegion`, `ReservedRange`, `FrameAllocator` (`src/subs/mem/`)

Hardware-coupled code (VGA writer, panic handler, boot asm) is not compiled here; live
checks happen through QEMU: `make vm agent`, then `make vm screen`.

Run it all:

```sh
make tests        # all host-side tests (cd tests && cargo test)
make build        # kernel
make iso test     # image-level validation (ELF + Multiboot2)
make vm agent && make vm screen   # boot the real image and read its screen
```

The `.cargo/config.toml` in this directory overrides the repo root's `[build] target`,
so these tests build for the host instead of `i686-runix-target`.
