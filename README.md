# Runix - Unix but Rust

`runix` (not runit, pronounced like R-Unix) is a hobby operating system kernel written
entirely in Rust, built to learn how a kernel boots, finds its memory, and starts
managing it. It currently targets 32-bit x86 (`i386`); 64-bit is planned.

The name is a play on *Unix*, but this is a from-scratch kernel, not a Unix clone. "R"
stands for Rust.

## Status

Early and minimal. On boot the kernel is loaded by GRUB over Multiboot2, hands control to
Rust, validates the boot information, builds a usable physical memory map from GRUB's,
reserves what is already in use, and smoke-tests a physical frame allocator. After that
it idles — no scheduler, userspace, filesystem, interrupts, or paging yet.

## Quick start

```bash
make build      # compile the kernel
make test       # validate the image is ELF + Multiboot2-compliant
make build_iso  # build runix.iso
make run        # boot it in QEMU
```

See [docs/build.md](docs/build.md) for requirements and toolchain details.

## Documentation

- [docs/architecture.md](docs/architecture.md) — codebase layout, subsystems, boot handoff.
- [docs/boot.md](docs/boot.md) — the Multiboot2 entry contract and tag format.
- [docs/build.md](docs/build.md) — requirements, commands, and toolchain constraints.
