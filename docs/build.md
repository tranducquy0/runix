# Building Runix

## Requirements

- Rust **nightly** (the build uses `cargo +nightly` and `-Z build-std`). Edition 2024.
- `grub-mkrescue` and `grub-file` (GRUB tooling) for building and validating the image.
- `qemu-system-i386` for running the ISO.
- `readelf` (binutils) for the image test.

## Build and run

```bash
make build      # compile the kernel with cargo +nightly (custom target i686-runix-target)
make iso test   # validate the image is ELF and Multiboot2-compliant
make iso build  # copy the kernel into iso/boot/rkrnl and build runix.iso with grub-mkrescue
make run        # boot runix.iso in QEMU (qemu-system-i386 -cdrom runix.iso, GTK window)
make tests      # run the host-side tests for the boot parser and memory modules
make clean      # cargo clean, remove runix.iso, the staged kernel and tests/target
make help       # list all targets
```

`iso` and `vm` are subcommand targets: the second word selects the action
(`make iso test`, `make iso build`, `make vm launch`, `make vm agent`,
`make vm screen`, `make vm stop`, `make vm foreground`). For headless runs,
`make vm agent` boots with a monitor socket, `make vm screen` decodes the
kernel's VGA text output through it (QEMU/GRUB takes ~10s before output
appears), and `make vm stop` shuts it down.

Build output lands at `target/i686-runix-target/debug/runix`.

Run `make iso test` after `make build` and before `make iso build` / `make run`: it enforces
that the kernel is a valid ELF binary and carries a valid Multiboot2 header, which is
what GRUB needs to boot it. The check itself lives in
[`scripts/kernel_image.sh`](../scripts/kernel_image.sh).

## Toolchain and constraints

- `no_std`, `no_main` i686 kernel; `panic = "abort"`.
- Custom target spec at [`.cargo/targets/i686-runix-target.json`](../.cargo/targets/i686-runix-target.json),
  selected by [`.cargo/config.toml`](../.cargo/config.toml), with linker flags for a
  static, non-PIE, `-nostdlib` build against [`scripts/linker.ld`](../scripts/linker.ld).
- The build uses `-Z build-std=core`
  `-Z build-std-features=compiler-builtins-mem` `-Z json-target-spec`, so it does not
  depend on a prebuilt `std` for the custom target.
- Only external dependency: [`spin`](https://crates.io/crates/spin) (for the VGA writer
  mutex).
- The binary disables `test` and `bench` ([`Cargo.toml`](../Cargo.toml)); the kernel is
  never exercised by `cargo test` directly. Use `make iso test` for image validation and
  `make tests` for the host-side test crate under [`tests/`](../tests), which re-exposes
  the parser and memory modules for testing on the host triple.
