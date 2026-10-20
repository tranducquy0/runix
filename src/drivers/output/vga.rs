//! VGA text-mode output driver — the kernel's print API.
//!
//! Writes to physical VGA memory at `0xb8000` (80x25 cells of
//! `[char, color]` byte pairs). The global [`WRITER`] is a `spin::Mutex`
//! because the kernel is single-threaded with interrupts off — any code can
//! `print!`/`println!` (macros below expand to `WRITER.lock()`).
//!
//! The cursor/buffer mechanics themselves ([`Writer`]) live in the sibling
//! `writer` module; this file is the locking + macro layer over it.
//!
//! Path note (per AGENTS.md): the macros below hardcode
//! `$crate::drivers::output::vga::WRITER`, so `WRITER` must stay in this
//! module — move only the writer mechanics if this splits further.

use spin::Mutex; // for avoiding some weird cases like multi mutable instances

// re-exported at the old path so callers keep `vga::Writer`
pub use writer::Writer;

mod writer;

/// Global screen cursor behind a `spin::Mutex`; the print macros lock it.
pub static WRITER: Mutex<Writer> = Mutex::new(Writer::new());

// export both print! and println! macros
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut writer = $crate::drivers::output::vga::WRITER.lock();
        write!(writer, $($arg)*).unwrap();
    });
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
