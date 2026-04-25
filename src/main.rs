#![no_main]
#![no_std]
use core::panic::PanicInfo;
use core::ptr::write_volatile;
#[unsafe(no_mangle)]
pub extern  "C" fn _start() -> ! {
    let vga = 0xb8000 as *mut (u8, u8);
    unsafe {
        write_volatile(vga.add(1), (b'O', 0x0f));
        write_volatile(vga.add(1), (b'K', 0x0f));
        };
    loop {};
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
const CHECKSUM:u32 = (0u32).wrapping_sub(0xE85250D6u32 + 0 + 24);
#[unsafe(link_section = ".multiboot2")]
#[used]
#[unsafe(no_mangle)]
pub static MULTIBOOT2_HEADER: [u32; 6] = [
    0xE85250D6, // magic
    0,          // architecture (i386)
    24,         // header length
    CHECKSUM, // checksum
    0, // end tag type
    8 // end tag size
];