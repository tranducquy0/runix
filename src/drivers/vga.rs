use core::fmt;

// writer struct
pub struct Writer {
    buffer: *mut u8,
    row: u8,
    collumn: u8,
    color: u8,
}
// writer implementation for fmt::Write
impl Writer {
    // init: set VGA buffer address, initial row, col and color
    pub const fn new() -> Self {
        Self { buffer: 0xb8000 as *mut u8, row: 0, collumn: 0, color: 0x0f }
    }
    // write_byte: send a byte and initial color to the VGA buffer
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                let idx = (self.row * 80 + self.collumn) *2;
                unsafe {
                    *self.buffer.add(idx as usize) = byte; // first cell: character
                    *self.buffer.add(idx as usize + 1) = self.color; // second cell: color (format 0x{background}{foreground})
                }
                self.collumn += 1;
                if (self.collumn >= 80) { // if current line have 80 cols, insert a new line
                    self.newline();
                }
            }
        }
    }
    // newline: insert a new blank line
    fn newline(&mut self) {
        self.collumn = 0;
        if self.row < 24 {
            // VGA buffer is 80x25, wrap to next line when exceeding width
            self.row += 1;
        } else {
            self.scroll();
        }
    }
    // scroll: scroll the "screen" (rewrite the buffer of "to" = "from")
    fn scroll(&mut self) {
        for row in 1..25 {
            for col in 0..80 {
                let from = (row * 80 + col) * 2;
                let to = ((row - 1) * 80 + col) * 2;

                unsafe {
                    *self.buffer.add(to) = *self.buffer.add(from);
                    *self.buffer.add(to +1) = *self.buffer.add(from + 1);
                }
            }
        }
        self.clear_row(24);
    }
    // clear a row (override all bytes in a row with spaces)
    fn clear_row(&mut self, row: usize) {
        for col in 0..80 {
            let idx = (row * 80 + col) * 2;

            unsafe {
                *self.buffer.add(idx) = b' ';
                *self.buffer.add(idx + 1) = self.color;
            }
        }
    }
    // write a string with write_byte
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe), // a square for weird characters
            }
        }
    }
}
// implement fmt::Write
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
pub static mut WRITER: Writer = Writer::new();

use core::fmt::Write;

// exports print! and println! macros (now we can use println! for printing! yay!)
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;

        unsafe {
            let writer = &mut *core::ptr::addr_of_mut!($crate::drivers::vga::WRITER);
            write!(writer, $($arg)*).unwrap();
        }
    });
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}