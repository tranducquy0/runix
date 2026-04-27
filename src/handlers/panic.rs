use core::panic::PanicInfo;
use crate::{drivers::vga, println};
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    let msg = info.message();
    println!("KERNEL PANIC! Something wrong (horribly wrong) happened!\n");
    println!("Stack: \n {}", msg);
    loop {}
}