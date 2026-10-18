---
name: kernel-build
description: Build, validate, and run the runix kernel. Use when compiling, testing the kernel image, creating the ISO, or booting in QEMU, or when the build/link fails.
---

## Build, validate, run

Exact order matters:

1. `make build` — compile kernel.
2. `make iso test` — validate the image (run before ISO/boot).
3. `make iso build` — pack `iso/` into `runix.iso` (only after `make iso test` passes).
4. `make run` — boot ISO in QEMU (GTK window). For agent/headless checks use `make vm agent` + `make vm screen` + `make vm stop`.

`make tests` runs host-side logic tests (`tests/` crate). `make clean` clears cargo artifacts + `runix.iso` + `iso/boot/rkrnl` + `tests/target`.

## What `make build` actually runs

It does not use a prebuilt std. It calls `cargo +nightly` with:

```
-Z build-std=core -Z build-std-features=compiler-builtins-mem -Z json-target-spec
```

- Target spec: `.cargo/targets/i686-runix-target.json` (32-bit i686, freestanding). Set as default in `.cargo/config.toml`.
- Output: `target/i686-runix-target/debug/runix`.
- Linker script: `scripts/linker.ld` (via `.cargo/config.toml` `rustflags`).

## Linker gotcha

The target spec sets `"linker": "ld.lld"`. If `ld.lld` is not on PATH, link fails with `linker ld.lld not found` even though compilation succeeds. The Rust toolchain ships `rust-lld`; override without editing config:

```
export CARGO_TARGET_I686_RUNIX_TARGET_LINKER=$(ls ~/.rustup/toolchains/nightly-*/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld | head -1)
```

## What `make iso test` checks

Runs `scripts/kernel_image.sh` on `target/i686-runix-target/debug/runix`. It requires `readelf` and `grub-file` on PATH:

- `readelf -h` — file is a real ELF.
- `grub-file --is-x86-multiboot2` — GRUB recognizes the Multiboot2 header.

If either tool is missing the script exits with an error; install `grub2-common` / `grub-common` for `grub-file`. A quick sanity check without grub: confirm the Multiboot2 magic `D6 50 52 E8` (LE) is present near the start of the ELF.

## No-test caveat

The binary is declared with `test = false, bench = false` in `Cargo.toml`. `cargo test` does nothing useful here; `make iso test` is the image gate and `make tests` runs the host-side logic tests (`tests/` crate).
