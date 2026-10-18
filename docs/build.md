# Building runix

## Requirements

- Rust **nightly** — the build runs `cargo +nightly` with `-Z build-std` (edition 2024).
- `grub-mkrescue` + `grub-file` (GRUB tooling) to pack and validate the image.
- `readelf` (binutils) — the other half of the image test.
- `qemu-system-i386` — running the ISO.
- `python3` — the `vm screen` / `vm stop` monitor helpers decode QEMU's replies with it.

## Build, validate, run

```bash
make build      # compile the kernel with cargo +nightly (custom target i686-runix-target)
make iso test   # validate the image is ELF and Multiboot2-compliant
make iso build  # copy the kernel into iso/boot/rkrnl, pack runix.iso with grub-mkrescue
make run        # boot runix.iso in QEMU (qemu-system-i386 -cdrom runix.iso, GTK window)
```

Run `make iso test` after every `make build`, before `make iso build` / `make run`: it
is the gate that proves the image has a valid ELF header and a Multiboot2 header GRUB
can actually boot. The check lives in
[`scripts/kernel_image.sh`](../scripts/kernel_image.sh) and fails loudly if `readelf` or
`grub-file` is missing. `make iso build` refuses to pack a missing kernel binary.

Build output lands at `target/i686-runix-target/debug/runix`.

## Testing and headless runs

```bash
make tests      # host-side tests for the boot parser and memory modules
make help       # list all targets
```

`iso` and `vm` are subcommand targets: the second word picks the action
(`make iso test|build`, `make vm launch|agent|foreground|screen|stop`).

- `make run` and `make vm launch` boot the ISO in a GTK window.
- `make vm agent` boots headless with a QEMU monitor on `.run/vm-monitor.sock` — that is
  how scripts and agents read the kernel without a display.
- `make vm screen` stops the CPU, dumps the 80×25 VGA text buffer over the monitor,
  decodes it, then resumes. QEMU/GRUB takes ~10s before kernel output exists, so poll it.
- `make vm foreground` is the headless launch without `daemonize`, for callers that want
  to own the monitor themselves.
- `make vm stop` quits the VM and drops `.run/`.

## Toolchain and constraints

- `no_std`, `no_main` i686 kernel; `panic = "abort"` in dev and release.
- Custom target spec at [`.cargo/targets/i686-runix-target.json`](../.cargo/targets/i686-runix-target.json),
  selected by [`.cargo/config.toml`](../.cargo/config.toml), linking against
  [`scripts/linker.ld`](../scripts/linker.ld) (static, non-PIE, `-nostdlib`).
- The build passes `-Z build-std=core` `-Z build-std-features=compiler-builtins-mem`
  `-Z json-target-spec`, so it never needs a prebuilt `std` for the custom target.
- Only external dependency: [`spin`](https://crates.io/crates/spin) (the VGA writer
  mutex).
- The binary disables `test` and `bench` ([`Cargo.toml`](../Cargo.toml)), so `cargo test`
  does nothing here. `make iso test` gates the image; `make tests` runs the host-side
  crate under [`tests/`](../tests), which re-exposes the kernel's logic modules on the
  host triple.

## Linker gotcha

The target spec sets `"linker": "ld.lld"`. If `ld.lld` is not on PATH the link fails
with `linker ld.lld not found` even though compilation succeeded. The Rust toolchain
ships `rust-lld`; override without editing config:

```bash
export CARGO_TARGET_I686_RUNIX_TARGET_LINKER=$(ls ~/.rustup/toolchains/nightly-*/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld | head -1)
```
