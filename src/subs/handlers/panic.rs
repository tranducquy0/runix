use core::panic::PanicInfo;
use crate::drivers;
use crate::println;

/// Kernel panic handler: print the panic message to VGA, then halt forever.
///
/// There is no unwinding (`panic = "abort"`) and no way to recover, so the
/// only job is surfacing enough context for a human watching the screen.
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    // The panic may itself strike while WRITER is held mid-output; under the
    // single-threaded, interrupts-disabled model the stale holder can never
    // resume, so drop it or the banner below deadlocks silently.
    unsafe {
        if drivers::output::vga::WRITER.is_locked() {
            drivers::output::vga::WRITER.force_unlock();
        }
    }

    let msg = info.message();
    println!("KERNEL PANIC! Something wrong (horribly wrong) happened!\n");
    println!("Stack: \n {}", msg);
    loop {}
}