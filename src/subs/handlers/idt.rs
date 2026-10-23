//! Interrupt Descriptor Table (IDT) and exception handling.
//!
//! Installs the kernel's IDT with handlers for CPU exceptions. Exceptions
//! are handled like panics - we print an error message and halt. Interrupts
//! remain disabled. The PIC is remapped separately in [`crate::drivers::pic`].
//!
//! This is minimal and meant to be extended later for IRQ handling.

#[cfg(target_arch = "x86")]
use core::arch::global_asm;

use crate::drivers;
use crate::println;
use crate::subs::arch::gdt;

/// Number of IDT entries (256 for x86).
const IDT_ENTRIES: usize = 256;

/// Kernel code segment selector (must match GDT).
const KERNEL_CODE_SELECTOR: u16 = gdt::KERNEL_CODE_SELECTOR;

/// IDT entry: represents a gate in the Interrupt Descriptor Table.
/// Each entry is 8 bytes.
#[derive(Clone, Copy)]
#[repr(C)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    type_attr: u8,
    offset_high: u16,
}

impl IdtEntry {
    const fn new() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            zero: 0,
            type_attr: 0,
            offset_high: 0,
        }
    }

    fn set_handler(&mut self, handler: u32) {
        self.offset_low = (handler & 0xFFFF) as u16;
        self.offset_high = ((handler >> 16) & 0xFFFF) as u16;
        self.selector = KERNEL_CODE_SELECTOR;
        self.zero = 0;
        // 0x8E = present (1) + DPL 0 (00) + storage segment 0 + gate type 1110 (interrupt gate)
        self.type_attr = 0x8E;
    }

    /// Stage a task gate: the CPU enters `selector`'s TSS with a full
    /// hardware task switch (own stack, own register file) instead of
    /// pushing an interrupt frame onto the suspect current stack. Used for
    /// the double fault, where SS:ESP cannot be trusted.
    fn set_task_gate(&mut self, selector: u16) {
        self.offset_low = 0;
        self.offset_high = 0;
        self.selector = selector;
        self.zero = 0;
        // 0x85 = present (1) + DPL 0 (00) + gate type 0101 (32-bit task gate)
        self.type_attr = 0x85;
    }
}

/// IDT pointer structure for lidt instruction.
#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u32,
}

/// The Interrupt Descriptor Table.
#[repr(C, align(16))]
struct InterruptDescriptorTable {
    entries: [IdtEntry; IDT_ENTRIES],
}

#[used]
static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable {
    entries: [IdtEntry::new(); IDT_ENTRIES],
};

#[cfg(target_arch = "x86")]
global_asm!(include_str!("../../asm/idt.asm"));

// External interrupt service routine stubs defined in assembly
#[cfg(target_arch = "x86")]
unsafe extern "C" {
    fn idt_load(idt_pointer_addr: u32);
    fn isr0();
    fn isr1();
    fn isr2();
    fn isr3();
    fn isr4();
    fn isr5();
    fn isr6();
    fn isr7();
    fn isr9();
    fn isr10();
    fn isr11();
    fn isr12();
    fn isr13();
    fn isr14();
    fn isr15();
    fn isr16();
    fn isr17();
    fn isr18();
    fn isr19();
    fn isr20();
    fn isr21();
    fn isr22();
    fn isr23();
    fn isr24();
    fn isr25();
    fn isr26();
    fn isr27();
    fn isr28();
    fn isr29();
    fn isr30();
    fn isr31();
}

/// Exception handler called from assembly stubs.
/// Behaves like the panic handler - prints and halts.
#[unsafe(no_mangle)]
extern "C" fn isr_handler(frame: *const u32) {
    // Uniform word layout at `frame` for error-code and no-code vectors
    // (see asm/idt.asm: every stub pushes the vector number, non-error
    // stubs push a dummy error word below it, and pusha lands below that):
    // [0..8) = pusha block (eax, ecx, edx, ebx, old esp, ebp, esi, edi)
    // [8] = vector, [9] = error code (0 for no-code vectors), [10] = eip
    let (vector, error_code, eip) =
        unsafe { (*frame.add(8), *frame.add(9), *frame.add(10)) };

    // A fault can land while the WRITER lock is held mid-output; under the
    // single-threaded, interrupts-disabled model the stale holder can never
    // resume, so drop it before re-locking or the banner below deadlocks.
    unsafe {
        if drivers::output::vga::WRITER.is_locked() {
            drivers::output::vga::WRITER.force_unlock();
        }
    }

    drivers::output::vga::WRITER.lock().clear_screen();
    println!("KERNEL PANIC! CPU EXCEPTION!\n");
    println!(
        "vector = {}, error code = {:#x}, eip = {:#x}",
        vector, error_code, eip
    );
    println!("System halted.");

    loop {}
}

/// Initialize the IDT and load it.
/// Must be called after GDT is initialized.
pub fn init() {
    let idt = &raw mut IDT;
    
    // Set up exception handlers
    unsafe {
        (*idt).entries[0].set_handler(isr0 as *const () as u32);
        (*idt).entries[1].set_handler(isr1 as *const () as u32);
        (*idt).entries[2].set_handler(isr2 as *const () as u32);
        (*idt).entries[3].set_handler(isr3 as *const () as u32);
        (*idt).entries[4].set_handler(isr4 as *const () as u32);
        (*idt).entries[5].set_handler(isr5 as *const () as u32);
        (*idt).entries[6].set_handler(isr6 as *const () as u32);
        (*idt).entries[7].set_handler(isr7 as *const () as u32);
        // Vector 8 (double fault) is a task gate, not an interrupt gate: the
        // CPU task-switches into subs/handlers/doublefault.rs's TSS so the
        // handler never runs on a possibly-broken stack.
        (*idt).entries[8].set_task_gate(gdt::KERNEL_DF_TSS_SELECTOR);
        (*idt).entries[9].set_handler(isr9 as *const () as u32);
        (*idt).entries[10].set_handler(isr10 as *const () as u32);
        (*idt).entries[11].set_handler(isr11 as *const () as u32);
        (*idt).entries[12].set_handler(isr12 as *const () as u32);
        (*idt).entries[13].set_handler(isr13 as *const () as u32);
        (*idt).entries[14].set_handler(isr14 as *const () as u32);
        (*idt).entries[15].set_handler(isr15 as *const () as u32);
        (*idt).entries[16].set_handler(isr16 as *const () as u32);
        (*idt).entries[17].set_handler(isr17 as *const () as u32);
        (*idt).entries[18].set_handler(isr18 as *const () as u32);
        (*idt).entries[19].set_handler(isr19 as *const () as u32);
        (*idt).entries[20].set_handler(isr20 as *const () as u32);
        (*idt).entries[21].set_handler(isr21 as *const () as u32);
        (*idt).entries[22].set_handler(isr22 as *const () as u32);
        (*idt).entries[23].set_handler(isr23 as *const () as u32);
        (*idt).entries[24].set_handler(isr24 as *const () as u32);
        (*idt).entries[25].set_handler(isr25 as *const () as u32);
        (*idt).entries[26].set_handler(isr26 as *const () as u32);
        (*idt).entries[27].set_handler(isr27 as *const () as u32);
        (*idt).entries[28].set_handler(isr28 as *const () as u32);
        (*idt).entries[29].set_handler(isr29 as *const () as u32);
        (*idt).entries[30].set_handler(isr30 as *const () as u32);
        (*idt).entries[31].set_handler(isr31 as *const () as u32);
    }
    
    let idt_ptr = IdtPointer {
        limit: ((core::mem::size_of::<IdtEntry>() * IDT_ENTRIES) as u16) - 1,
        base: &raw const IDT as u32,
    };
    
    unsafe {
        idt_load(&raw const idt_ptr as u32);
    }
    
    println!("installed kernel idt");
}
