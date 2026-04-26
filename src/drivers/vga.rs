struct VGAChar {
    ascii: u8,
    color: u8,
}

const VGA_BUFFER: *mut VGAChar = 0xb8000 as *mut VGAChar;

pub fn write_byte(offset: usize, byte: u8) {
    unsafe {
        core::ptr::write_volatile(
            VGA_BUFFER.add(offset),
            VGAChar { ascii: byte, color: 0x0f },
        );
    }
}
pub fn vga_print(s: &str) {
    let bytes = s.as_bytes();
    for (idx, &b) in bytes.iter().enumerate() {
        write_byte(idx, b);
    }
}