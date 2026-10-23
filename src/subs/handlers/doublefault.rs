//! Double fault (#DF, vector 8) handling via a dedicated task.
//!
//! #DF is special on i686: the CPU may be faulting *because* SS:ESP is
//! suspect, so an ordinary interrupt gate would push a stack frame through
//! the very stack that is broken. The i686-correct move is a task gate in
//! IDT entry 8 pointing at a second TSS (registered in the kernel GDT at
//! selector `0x20`): entry performs a full hardware task switch, landing
//! this handler on its own stack with its own selector set — independent of
//! whatever state the interrupted task left behind.
//!
//! The task is never entered by `ltr` and stays "available" until #DF fires,
//! at which point hardware marks it busy and snapshots the interrupted
//! context into the kernel TSS (selector `0x18`) — that snapshot is what the
//! handler reports before halting, same contract as [`crate::subs::handlers::idt`]
//! and the panic handler: print, then stop.

use crate::drivers;
use crate::println;
use crate::subs::arch::{gdt, tss::{self, Tss}};

/// 4 KiB reserved for the double-fault task's stack, 16-byte aligned for a
/// sane call stack. The bytes stay untouched by Rust — only the hardware
/// task switch and the handler's framed calls run there — so the field never
/// gets "read" from Rust's point of view.
const STACK_SIZE: usize = 0x1000;

#[allow(dead_code)]
#[repr(align(16))]
struct DoubleFaultStack([u8; STACK_SIZE]);

#[used]
static mut DF_STACK: DoubleFaultStack = DoubleFaultStack([0; STACK_SIZE]);

/// The task's register file, zero-filled so any field we forget stages as 0.
#[used]
static mut DF_TSS: Tss = Tss::zeroed();

/// Linear address of the double-fault TSS, consumed by [`gdt::init`].
pub fn tss_address() -> u32 {
    &raw const DF_TSS as u32
}

/// The interrupted task's context, as hardware saved it in the kernel TSS.
/// Valid only after the task gate has actually fired.
extern "C" fn doublefault_handler() -> ! {
    let eip = tss::saved_eip();
    let esp = tss::saved_esp();

    drivers::output::vga::WRITER.lock().clear_screen();
    println!("KERNEL PANIC! Something wrong (horribly wrong) happened!\n");
    println!("double fault (#df): faulting task eip = {:#x}, esp = {:#x}", eip, esp);
    println!("System halted.");

    loop {}
}

/// Fill the double-fault task's register file. Safe to call any time before
/// a double fault; the GDT descriptor only needed the address, this is what
/// makes the switch survive. Interrupts stay disabled per the segment/flag
/// choices below.
pub fn init() {
    let tss_ptr = &raw mut DF_TSS;
    let stack_top = (&raw mut DF_STACK as u32) + STACK_SIZE as u32;

    unsafe {
        (*tss_ptr).eip = doublefault_handler as *const () as u32;
        (*tss_ptr).esp = stack_top;
        (*tss_ptr).cs = gdt::KERNEL_CODE_SELECTOR;
        (*tss_ptr).ss = gdt::KERNEL_DATA_SELECTOR;
        (*tss_ptr).es = gdt::KERNEL_DATA_SELECTOR;
        (*tss_ptr).ds = gdt::KERNEL_DATA_SELECTOR;
        (*tss_ptr).fs = gdt::KERNEL_DATA_SELECTOR;
        (*tss_ptr).gs = gdt::KERNEL_DATA_SELECTOR;
        // Reserved bit 1 set, IF clear: no interrupts inside the handler.
        (*tss_ptr).eflags = 0x00000002;
        // Paging does not exist yet; stage the real value once it does.
        (*tss_ptr).cr3 = 0;
    }

    println!(
        "staged double fault task: selector = {:#x}, eip = {:#x}, stack top = {:#x}",
        gdt::KERNEL_DF_TSS_SELECTOR,
        doublefault_handler as *const () as u32,
        stack_top
    );
}
