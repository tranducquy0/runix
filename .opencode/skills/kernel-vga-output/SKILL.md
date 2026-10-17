---
name: kernel-vga-output
description: VGA text output for runix — WRITER mutex, print/println macros, and the output driver path. Use when changing screen output, the print macros, or adding output drivers.
---

## Layout

- `src/drivers/output/vga.rs` — VGA text-mode driver (`Writer`, `WRITER`, `print!`/`println!`).
- `src/drivers/output/` — group for output drivers (VGA now; framebuffer/serial later).
- `src/drivers/input/` — placeholder for input drivers (`keyboard.rs` comes later).
- `src/drivers/mod.rs` — `pub mod input; pub mod output;`

## Writer

- Writes to the legacy VGA text buffer at physical `0xb8000` (80x25). Each cell is 2 bytes: char then color. Color byte format `0x{bg}{fg}` (default `0x0f` = white on black).
- `spin::Mutex<Writer>` as `pub static WRITER` — no interrupts yet, so a spin lock is the whole synchronization story.
- `unsafe impl Send for Writer {}` and `Sync` are required so the `static Mutex<Writer>` is allowed; don't remove.
- Handles `\n`, column overflow (newline), and scrolling (shifts rows up, clears last row). Non-ASCII bytes (outside `0x20..=0x7e`) render as `0xfe` (a block).

## Macros — path is load-bearing

`print!`/`println!` are `#[macro_export]`, so they live at the crate root and are available everywhere (including the panic handler). Their bodies hardcode the driver path:

```rust
let mut writer = $crate::drivers::output::vga::WRITER.lock();
```

If you move or rename the VGA module, update this `$crate::...` path or every call site fails to compile. `main.rs` keeps `#[macro_use] mod drivers;` — leave it as is.

## Panic handler

`src/subs/handlers/panic.rs` uses `println!` to dump the panic message. It must not allocate or lock anything beyond the existing VGA `WRITER` mutex.

## Adding drivers

- Output drivers go under `src/drivers/output/` (add `mod.rs` entry + `pub mod yourdriver;`).
- Input drivers go under `src/drivers/input/` (currently an empty placeholder; add `keyboard.rs` there when it lands).
- Keep `input`/`output` as the split; don't add new top-level driver groups.
