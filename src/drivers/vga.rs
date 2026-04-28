use core::fmt;
use spin::Mutex;

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

pub struct Writer {
    buffer: *mut u8,
    row: usize,
    column: usize,
    color: u8,
}

impl Writer {
    pub const fn new() -> Self {
        Self {
            buffer: 0xb8000 as *mut u8,
            row: 0,
            column: 0,
            color: 0x0f,
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                if self.column >= BUFFER_WIDTH {
                    self.newline();
                }

                if self.row >= BUFFER_HEIGHT {
                    self.scroll();
                }

                let idx = (self.row * BUFFER_WIDTH + self.column) * 2;

                unsafe {
                    *self.buffer.add(idx) = byte;
                    *self.buffer.add(idx + 1) = self.color;
                }

                self.column += 1;
            }
        }
    }

    fn newline(&mut self) {
        self.column = 0;

        if self.row < BUFFER_HEIGHT - 1 {
            self.row += 1;
        } else {
            self.scroll();
        }
    }

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

    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            let idx = (row * BUFFER_WIDTH + col) * 2;

            unsafe {
                *self.buffer.add(idx) = b' ';
                *self.buffer.add(idx + 1) = self.color;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
}

unsafe impl Send for Writer {}
unsafe impl Sync for Writer {}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub static WRITER: Mutex<Writer> = Mutex::new(Writer::new());

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut writer = $crate::drivers::vga::WRITER.lock();
        write!(writer, $($arg)*).unwrap();
    });
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}