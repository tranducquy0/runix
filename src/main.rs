// disable normal features that can't build a kernel
#![no_main]
#![no_std]

use core::arch::global_asm;

mod boot;
// driver and handler module
#[macro_use] // use the macros from vga module (KEEP IT AS IS, don't ask why!)
mod drivers;

mod handlers;
// memory manager
mod memory;
use boot::multiboot2::{self, BootInformation};

global_asm!(
    r#"
    .section .bss
    .align 16
boot_stack:
    .skip 4096
boot_stack_top:

    .section .text
    .global _start
_start:
    mov esp, offset boot_stack_top
    push ebx
    push eax
    call kernel_main
1:
    hlt
    jmp 1b
"#
);

#[unsafe(no_mangle)]
extern "C" fn kernel_main(multiboot_magic: u32, multiboot_info_addr: u32) -> ! {
    println!("Welcome to the R-Unix kernel");

    if multiboot_magic != multiboot2::BOOTLOADER_MAGIC {
        println!("invalid multiboot2 magic: {:#x}", multiboot_magic);
        loop {}
    }

    let grub_boot_info = unsafe { BootInformation::load(multiboot_info_addr as usize) };
    let Some(mem_map_tag) = grub_boot_info.memory_map_tag() else {
        println!("GRUB did not provide a memory map tag");
        loop {}
    };

    for region in mem_map_tag.memory_areas() {
        if region.is_available() {
            println!(
                "usable: start = {:#x}, len = {:#x}",
                region.start_address(),
                region.size()
            );
        }
    }
    loop {}
}

// multiboot2 header checksum
const MULTIBOOT2_HEADER_MAGIC: u32 = 0xE85250D6;
const MULTIBOOT2_ARCHITECTURE_I386: u32 = 0;
const MULTIBOOT2_HEADER_LENGTH: u32 = 40;
const MULTIBOOT2_MEMORY_MAP_TAG: u32 = 6;
const CHECKSUM: u32 = (0u32).wrapping_sub(
    MULTIBOOT2_HEADER_MAGIC + MULTIBOOT2_ARCHITECTURE_I386 + MULTIBOOT2_HEADER_LENGTH,
);


// multboot2 header struct
#[repr(C, align(8))]
struct MultibootHeader {
    data: [u32; 10],
}

// multiboot2 header (for real this time)
#[unsafe(link_section = ".multiboot2")]
#[used]
#[unsafe(no_mangle)]
static MULTIBOOT2_HEADER: MultibootHeader = MultibootHeader {
    data: [
        MULTIBOOT2_HEADER_MAGIC,
        MULTIBOOT2_ARCHITECTURE_I386,
        MULTIBOOT2_HEADER_LENGTH,
        CHECKSUM,
        1, // information request tag type
        0, // required
        16,
        MULTIBOOT2_MEMORY_MAP_TAG,
        0, // end tag type
        8, // end tag size
    ],
};
