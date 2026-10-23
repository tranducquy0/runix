//! 8259 Programmable Interrupt Controller driver.
//!
//! Remaps the master and slave PICs so IRQs don't collide with CPU exceptions.
//! Master is remapped to offset 0x20, slave to offset 0x28. All IRQs are
//! masked initially; drivers will unmask them as needed.

use core::arch::asm;

const PIC1_CMD: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const PIC_EOI: u8 = 0x20;

#[inline]
unsafe fn outb(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
    }
}

#[inline]
unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Initialize and remap the 8259 PIC.
/// Master -> interrupts 0x20-0x27, Slave -> interrupts 0x28-0x2F.
/// All IRQs are masked after initialization.
pub fn init() {
    // ICW1: start initialization
    unsafe {
        outb(PIC1_CMD, 0x11);
        outb(PIC2_CMD, 0x11);
        // ICW2: vector offsets
        outb(PIC1_DATA, 0x20);
        outb(PIC2_DATA, 0x28);
        // ICW3: tell master PIC there is a slave PIC at IRQ2
        outb(PIC1_DATA, 0x04);
        outb(PIC2_DATA, 0x02);
        // ICW4: 8086 mode
        outb(PIC1_DATA, 0x01);
        outb(PIC2_DATA, 0x01);
        // Mask all IRQs
        outb(PIC1_DATA, 0xFF);
        outb(PIC2_DATA, 0xFF);
    }
}

/// Send End of Interrupt signal to the PIC(s).
pub fn send_eoi(irq: u8) {
    if irq >= 8 {
        unsafe {
            outb(PIC2_CMD, PIC_EOI);
        }
    }
    unsafe {
        outb(PIC1_CMD, PIC_EOI);
    }
}

/// Mask a specific IRQ.
pub fn mask_irq(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let irq = if irq < 8 { irq } else { irq - 8 };
    unsafe {
        let val = inb(port);
        outb(port, val | (1 << irq));
    }
}

/// Unmask a specific IRQ.
pub fn unmask_irq(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let irq = if irq < 8 { irq } else { irq - 8 };
    unsafe {
        let val = inb(port);
        outb(port, val & !(1 << irq));
    }
}
