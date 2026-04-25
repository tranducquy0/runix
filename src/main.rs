#![no_main]
#![no_std]
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern  "C" fn _start() -> ! {
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