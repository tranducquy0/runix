#![no_main]
#![no_std]
use core::panic::PanicInfo;

pub extern  "C" fn _start() -> ! {
    loop {};
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
const CHECKSUM:u32 = (0u32).wrapping_sub(0xE85250D6u32 + 0 +32);
#[unsafe(link_section = ".multiboot2")]
#[unsafe(no_mangle)]
pub static MULTIBOOT2_HEADER: [u32; 8] = [
    0xE85250D6, // magic
    0,          // architecture (i386)
    32,         // header length
    CHECKSUM, // checksum
    0, 0, 0, 0, // end tag
];