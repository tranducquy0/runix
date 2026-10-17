# AGENTS.md

## Commands (exact)
- `make build` — build kernel via `cargo +nightly` with `-Z build-std=core -Z build-std-features=compiler-builtins-mem -Z json-target-spec` for target `i686-runix-target`. Outputs `target/i686-runix-target/debug/runix`.
- `make build_iso` — copies kernel to `iso/boot/rkrnl` and runs `grub-mkrescue -o runix.iso iso/`.
- `make run` — boots ISO in QEMU: `qemu-system-i386 -cdrom runix.iso`.
- `make test` — validates kernel image via `scripts/kernel_image.sh` (checks ELF with `readelf` and Multiboot2 with `grub-file --is-x86-multiboot2`). Must have both tools installed.
- `make clean` — `cargo clean` + remove `runix.iso` and `iso/boot/rkrnl`.

## Toolchain / constraints
- Rust nightly required (Makefile uses `cargo +nightly`). Edition 2024. `panic = "abort"` in dev/release.
- `no_std`, `no_main` i686 kernel. Custom target `i686-runix-target` is referenced; build uses `-Z build-std` (does not rely on prebuilt std for that target).
- Only dependency: `spin` (`Cargo.toml`).
- Binary `runix` at `src/main.rs` with `test = false`, `bench = false` — do not expect `cargo test` to exercise kernel code; use `make test` for image validation.

## Architecture / entrypoints
- `src/main.rs` — kernel entry: Multiboot2 header embedded, `kernel_main(multiboot_magic, multiboot_info_addr)`, boot memory reservation, frame allocator smoke test. Boot stub is `global_asm!(include_str!("asm/boot.asm"))`.
- `src/asm/boot.asm` — raw boot assembly (keep register/stack handoff to GRUB explicit).
- Subsystems under `src/subs/` (declared in `src/subs/mod.rs`): `boot/` (Multiboot2 types/helpers), `mem/` (boot memory map + frame allocator; renamed from `memory`), `handlers/` (panic handler; future IDT/interrupt dispatch).
- Drivers under `src/drivers/` split into `output/` (VGA text driver) and `input/` (placeholder for keyboard). `main.rs` keeps `#[macro_use] mod drivers;` so the `print!`/`println!` macros stay available; their bodies hardcode `$crate::drivers::output::vga::WRITER` — update that path if the module moves.
- Linker: `scripts/linker.ld`. ISO layout: `iso/boot/` + GRUB config under `iso/`.

## Skills
Repo-local agent skills live in `.opencode/skills/` (`kernel-build`, `kernel-memory`, `kernel-multiboot2`, `kernel-vga-output`). Load one via the `skill` tool when working in that area; `kernel-build` covers toolchain gotchas (e.g. the `ld.lld` / `rust-lld` override).

## Verification notes
- After building, run `make test` before `make build_iso`/`make run` — it enforces the image is valid ELF and Multiboot2-compliant.
- Artifacts ignored: `/target`, `iso/boot/rkrnl`, `iso/boot/kernel.bin`, `/*.iso`.