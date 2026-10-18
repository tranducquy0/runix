//! Hardware drivers, split by direction: `output` (screen) and `input`
//! (keyboard, placeholder). The `print!`/`println!` macros exported by the
//! VGA driver are re-exported crate-wide via `#[macro_use]` in `main.rs`.

pub mod input;
pub mod output;
