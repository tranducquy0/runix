//! VGA text-mode output driver.
//!
//! Writes to physical VGA memory at `0xb8000` (80x25 cells of
//! `[char, color]` byte pairs). The global [`WRITER`] is a `spin::Mutex`
//! because the kernel is single-threaded with interrupts off — any code can
//! `print!`/`println!` (macros below expand to `WRITER.lock()`).
//!
//! Cursor state (`row`, `column`) lives in the [`Writer`] and wraps by
//! scrolling: the oldest line is dropped whenever the bottom row is
//! written past. Non-printable bytes render as `0xfe` blocks.

use core::fmt;
use spin::Mutex; // for avoiding some weird cases like multi mutable instances
// buffer width and size
const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

/// Stateful cursor + buffer pointer for one VGA text screen.
///
/// All methods are `&mut self` and assume the caller holds [`WRITER`]'s
/// lock; `buffer` is a raw pointer to `0xb8000` rather than a reference
/// because it points at memory-mapped I/O, not Rust-managed storage.
pub struct Writer {
    buffer: *mut u8, // 0xb8000 - the OG VGA addr
    row: usize, // keeping track of the current row
    column: usize, // (like above, but column)
    color: u8, // color (format: 0x{bg}{fg}, eg. 0x0f (white text on black))
}

impl Writer {
    // Empty cursor at the top-left, white on black.
    pub const fn new() -> Self { // new instance of writer struct
        Self {
            buffer: 0xb8000 as *mut u8,
            row: 0,
            column: 0,
            color: 0x0f,
        }
    }

    // Put one printable byte at the cursor, advancing/wrapping/scrolling as needed.
    fn write_byte(&mut self, byte: u8) { // receive a byte (type: u8), check it, then write it to VGA memory
        match byte {
            b'\n' => self.newline(),
            byte => {
                // overflow checks
                if self.column >= BUFFER_WIDTH {
                    self.newline();
                }

                if self.row >= BUFFER_HEIGHT {
                    self.scroll();
                }

                // get index of cursor
                let idx = (self.row * BUFFER_WIDTH + self.column) * 2;

                unsafe {
                    *self.buffer.add(idx) = byte;
                    *self.buffer.add(idx + 1) = self.color;
                }

                self.column += 1;
            }
        }
    }

    // Move to column 0 of the next row, or scroll when already on the last row.
    fn newline(&mut self) {
        self.column = 0;

        if self.row < BUFFER_HEIGHT - 1 {
            self.row += 1;
        } else {
            self.scroll();
        }
    }

    // Shift every row up one, blank the bottom row, park cursor at line start.
    fn scroll(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let from = (row * BUFFER_WIDTH + col) * 2;
                let to = ((row - 1) * BUFFER_WIDTH + col) * 2;
                unsafe {
                    let ch = *self.buffer.add(from);
                    let color = *self.buffer.add(from + 1);

                    *self.buffer.add(to) = ch;
                    *self.buffer.add(to + 1) = color;
                }
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);

        self.row = BUFFER_HEIGHT - 1;
        self.column = 0;
    }

    // Fill one row with spaces in the current color.
    fn clear_row(&mut self, row: usize) { // "clear" a row (fill it with spaces)
        for col in 0..BUFFER_WIDTH {
            let idx = (row * BUFFER_WIDTH + col) * 2;

            unsafe {
                *self.buffer.add(idx) = b' ';
                *self.buffer.add(idx + 1) = self.color;
            }
        }
    }

    // Blank every row and reset the cursor to the top-left (used at boot).
    pub fn clear_screen(&mut self) { // wipe the whole buffer and reset the cursor
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }

        self.row = 0;
        self.column = 0;
    }

    /**
     * write a string to the vga
     * work fine for ASCII characters (0x20..0x7e)
     * print a square for non-ASCII characters
    */
    // Render a str byte-by-byte: printable ASCII and '\n' pass through, everything else becomes 0xfe.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
}

// stub code (to use Mutex), DON'T TOUCH
unsafe impl Send for Writer {}
unsafe impl Sync for Writer {}

// implement fmt::Write
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

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