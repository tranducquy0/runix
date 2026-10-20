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
make setup android|arch  # first-time setup: install the toolchain (Termux / Arch)
make build      # compile the kernel (cargo +nightly, custom i686 target)
make iso test   # validate the image is ELF + Multiboot2-compliant
make iso build  # pack iso/ into runix.iso (grub-mkrescue)
make run        # boot runix.iso in QEMU (GTK window)
```

The order matters: `make iso test` sits between `build` and `iso build` / `run` because
it enforces the image GRUB needs to boot.

Headless/agent flow and the remaining targets:

```bash
make tests      # host-side tests for the boot parser and memory modules
make vm agent   # boot headless with a monitor socket (for scripts/agents)
make vm screen  # dump the kernel's VGA text output via the monitor
make vm stop    # stop the headless VM
make help       # list every target
```

`make vm screen` needs ~10s of grace after `make vm agent` — QEMU and GRUB boot before
the kernel prints anything.

See [docs/build.md](docs/build.md) for requirements and toolchain details.

## Documentation

- [docs/architecture.md](docs/architecture.md) — codebase layout, subsystems, boot handoff.
- [docs/boot.md](docs/boot.md) — the Multiboot2 entry contract and tag format.
- [docs/build.md](docs/build.md) — requirements, commands, and toolchain constraints.
