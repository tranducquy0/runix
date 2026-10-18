use core::panic::PanicInfo;
use crate::println;

/// Kernel panic handler: print the panic message to VGA, then halt forever.
///
/// There is no unwinding (`panic = "abort"`) and no way to recover, so the
/// only job is surfacing enough context for a human watching the screen.
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    let msg = info.message();
    println!("KERNEL PANIC! Something wrong (horribly wrong) happened!\n");
    println!("Stack: \n {}", msg);
    loop {}
}