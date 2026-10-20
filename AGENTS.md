# AGENTS.md

## Commands (exact)
- `make build` — build kernel via `cargo +nightly` with `-Z build-std=core -Z build-std-features=compiler-builtins-mem -Z json-target-spec` for target `i686-runix-target`. Outputs `target/i686-runix-target/debug/runix`. On systems without rustup (Termux) the Makefile falls back to `RUSTC_BOOTSTRAP=1 cargo build` with the distro stable + rust-src.
- `make setup android|arch` — install the dev toolchain (dispatcher: `scripts/setup.sh`; Termux via `pkg`, Arch via `pacman` + rustup nightly).
- `make iso build` — copies kernel to `iso/boot/rkrnl` and runs `grub-mkrescue -o runix.iso iso/` (dispatcher: `scripts/iso.sh`; does not rebuild the kernel by itself).
- `make run` — boots ISO in QEMU with a visible GTK window.
- `make iso test` — validates kernel image via `scripts/kernel_image.sh` (checks ELF with `readelf` and Multiboot2 with `grub-file --is-x86-multiboot2`). Must have both tools installed.
- `make tests` — runs the host-side kernel module tests (`tests/` crate, `cargo test` for x86_64 host).
- `make vm agent` — boots the ISO headless in QEMU with a monitor at `.run/vm-monitor.sock`; `make vm screen` dumps the kernel's VGA text; `make vm stop` kills the VM (dispatcher: `scripts/vm.sh`). QEMU/GRUB takes ~10s before kernel output appears — poll `make vm screen`.
- `make clean` — `cargo clean` + remove `runix.iso`, `iso/boot/rkrnl`, and `tests/target`.

## Toolchain / constraints
- Rust nightly required (Makefile uses `cargo +nightly`). Edition 2024. `panic = "abort"` in dev/release.
- `no_std`, `no_main` i686 kernel. Custom target `i686-runix-target` is referenced; build uses `-Z build-std` (does not rely on prebuilt std for that target).
- Only dependency: `spin` (`Cargo.toml`).
- Binary `runix` at `src/main.rs` with `test = false`, `bench = false` — do not expect `cargo test` to exercise kernel code; use `make tests` (host logic tests) and `make iso test` (image validation).

## Architecture / entrypoints
- `src/main.rs` — kernel entry: Multiboot2 header embedded, `kernel_main(multiboot_magic, multiboot_info_addr)` (clears the VGA screen first), boot memory reservation, frame allocator smoke test. Boot stub is `global_asm!(include_str!("asm/boot.asm"))`.
- `src/asm/boot.asm` — raw boot assembly (keep register/stack handoff to GRUB explicit).
- Subsystems under `src/subs/` (declared in `src/subs/mod.rs`): `boot/` (Multiboot2 types/helpers), `mem/` (boot memory map + frame allocator; renamed from `memory`), `handlers/` (panic handler; future IDT/interrupt dispatch).
- Drivers under `src/drivers/` split into `output/` (VGA text driver) and `input/` (placeholder for keyboard). `main.rs` keeps `#[macro_use] mod drivers;` so the `print!`/`println!` macros stay available; their bodies hardcode `$crate::drivers::output::vga::WRITER` — update that path if the module moves.
- Linker: `scripts/linker.ld`. ISO layout: `iso/boot/` + GRUB config (`iso/boot/grub/grub.cfg`, `timeout=4`) under `iso/`.
- Host tests: `tests/runix-tests` crate re-exposes kernel logic modules (`subs/boot/multiboot2`, `subs/mem`) via thin `include!`/`#[path]` shims in `tests/src/subs/`; hardware-coupled code (VGA, boot asm) is covered by the QEMU flow instead.

## Skills
Repo-local agent skills live in `.opencode/skills/` (`kernel-build`, `kernel-memory`, `kernel-multiboot2`, `kernel-vga-output`, `commit`, `code-review`). Load one via the `skill` tool when working in that area; `kernel-build` covers toolchain details (e.g. `i686-runix-target` and LLD linking), `commit` codifies the Conventional Commits style used in this repo, and `code-review` governs PR/diff reviews (evidence-based findings, severity rules, read-only scope).

## Verification notes
- After building, run `make iso test` before `make iso build`/`make run` — it enforces the image is valid ELF and Multiboot2-compliant.
- To verify live boot behavior, `make vm agent; sleep 12; make vm screen; make vm stop` and check the kernel output on the dumped screen.
- Artifacts ignored: `/target`, `iso/boot/rkrnl`, `iso/boot/kernel.bin`, `/*.iso`, `/tests/target`, `/.run/`.