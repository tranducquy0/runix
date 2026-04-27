// disable normal features that can't build a kernel
#![no_main]
#![no_std]

// driver and handler module
#[macro_use] // use the macros from vga module (KEEP IT AS IS, don't ask why!)
mod drivers;

mod handlers;
// memory manager
mod memory;

// stack implementation
#[repr(align(16))]
struct Stack([u8; 4096]);
static mut STACK: Stack = Stack([0; 4096]);

// entry point
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        let stack_top =
            ((core::ptr::addr_of!(STACK) as usize) + 4096) & !0xF;

        core::arch::asm!(
            "mov esp, {}",
            in(reg) stack_top
        );
    }
    unsafe {
        println!("Welcome to the R-Unix Kernel!")
    }
    loop {}
}

// multiboot2 header checksum
const CHECKSUM:u32 = (0u32).wrapping_sub(0xE85250D6u32 + 0 + 24);

// multboot2 header struct
#[repr(C, align(8))]
struct MultibootHeader {
    data: [u32; 6],
}

// multiboot2 header (for real this time)
#[unsafe(link_section = ".multiboot2")]
#[used]
#[unsafe(no_mangle)]
static MULTIBOOT2_HEADER: MultibootHeader = MultibootHeader {
    data: [
        0xE85250D6, // magic
        0,          // architecture
        24,         // length
        (0u32).wrapping_sub(0xE85250D6 + 0 + 24),
        0,          // end tag type
        8,          // end tag size
    ],
};