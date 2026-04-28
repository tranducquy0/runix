use core::fmt;
use spin::Mutex; // for avoiding some weird cases like multi mutable instances
// buffer width and size
const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

// writer struct
pub struct Writer {
    buffer: *mut u8, // 0xb8000 - the OG VGA addr
    row: usize, // keeping track of the current row
    column: usize, // (like above, but column)
    color: u8, // color (format: 0x{bg}{fg}, eg. 0x0f (white text on black))
}

impl Writer {
    pub const fn new() -> Self { // new instance of writer struct
        Self {
            buffer: 0xb8000 as *mut u8,
            row: 0,
            column: 0,
            color: 0x0f,
        }
    }

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

    fn clear_row(&mut self, row: usize) { // "clear" a row (fill it with spaces)
        for col in 0..BUFFER_WIDTH {
            let idx = (row * BUFFER_WIDTH + col) * 2;

            unsafe {
                *self.buffer.add(idx) = b' ';
                *self.buffer.add(idx + 1) = self.color;
            }
        }
    }

    /**
    write a string 
    input: a string
    processing pipeline: loop through the string's bytes 
    (type u8) and write it to the VGA, prints 0xfe (a square)
    if detecting any weird character
    */
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

// create WRITER Mutex instance
pub static WRITER: Mutex<Writer> = Mutex::new(Writer::new());

// export both print! and println! macros
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