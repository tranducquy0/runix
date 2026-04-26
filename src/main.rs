#![no_main]
#![no_std]
use core::panic::PanicInfo;
use core::ptr::write_volatile;

#[repr(align(16))]
struct Stack([u8; 4096]);
static mut STACK: Stack = Stack([0; 4096]);

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
    #[repr(C)]
    struct VGAChar {
        _char: u8,
        color: u8
    }
    let vga = 0xb8000 as *mut VGAChar;
    unsafe {
        core::ptr::write_volatile(vga.add(0), VGAChar { _char: b'O', color: 0x0f});
        core::ptr::write_volatile(vga.add(1), VGAChar { _char: b'K', color: 0x0f });
    }
    loop {}
}
// simple panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
// multiboot2 header
const CHECKSUM:u32 = (0u32).wrapping_sub(0xE85250D6u32 + 0 + 24);

#[repr(C, align(8))]
struct MultibootHeader {
    data: [u32; 6],
}

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