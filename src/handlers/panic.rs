use core::panic::PanicInfo;
use crate::drivers::vga;
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    vga::vga_print("KERNEL PANIC! Something wrong (horribly wrong) happened!");
    let msg = info.message();
    loop {}
}