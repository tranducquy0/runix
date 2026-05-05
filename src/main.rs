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
use memory::{allocator::FrameAllocator, BootMemoryMap, ReservedRange};

global_asm!(
    include_str!("asm/boot.asm")
);

unsafe extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
    static boot_stack: u8;
    static boot_stack_top: u8;
}

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

    let Some(mut memory_map) = BootMemoryMap::from_multiboot(mem_map_tag.memory_areas()) else {
        println!("memory map did not fit in the kernel boot map");
        loop {}
    };

    // These ranges are occupied before the allocator ever runs, so we cut them
    // out of the GRUB memory map first. That keeps later allocation logic from
    // accidentally handing out frames that belong to the kernel or bootloader.
    let reserved_ranges = [
        ReservedRange::new(
            core::ptr::addr_of!(__kernel_start) as usize,
            core::ptr::addr_of!(__kernel_end) as usize,
        ),
        ReservedRange::new(
            grub_boot_info.start_address(),
            grub_boot_info.end_address(),
        ),
        ReservedRange::new(
            core::ptr::addr_of!(boot_stack) as usize,
            core::ptr::addr_of!(boot_stack_top) as usize,
        ),
    ];

    for reserved in reserved_ranges {
        if !memory_map.reserve(reserved) {
            println!("reserved range split the boot map too many times");
            loop {}
        }
    }

    println!("usable memory after reservations:");
    for region in memory_map.regions() {
        println!(
            "usable: start = {:#x}, len = {:#x}",
            region.start(),
            region.size()
        );
    }

    let mut allocator = FrameAllocator::new(&memory_map);
    println!("first few allocatable frames:");
    for _ in 0..4 {
        match allocator.alloc() {
            Some(frame) => println!("frame = {:#x}", frame),
            None => {
                println!("no more frames");
                break;
            }
        }
    }

    loop {}
}

// multiboot2 header checksum
const MULTIBOOT2_HEADER_MAGIC: u32 = 0xE85250D6;
const MULTIBOOT2_ARCHITECTURE_I386: u32 = 0;
const MULTIBOOT2_HEADER_LENGTH: u32 = 40;
const CHECKSUM: u32 = (0u32).wrapping_sub(
    MULTIBOOT2_HEADER_MAGIC + MULTIBOOT2_ARCHITECTURE_I386 + MULTIBOOT2_HEADER_LENGTH,
);


#[repr(C, align(8))]
struct MultibootHeader {
    data: [u8; MULTIBOOT2_HEADER_LENGTH as usize],
}

#[unsafe(link_section = ".multiboot2")]
#[used]
#[unsafe(no_mangle)]
static MULTIBOOT2_HEADER: MultibootHeader = MultibootHeader {
    data: [
        0xd6, 0x50, 0x52, 0xe8, // magic
        0x00, 0x00, 0x00, 0x00, // architecture
        0x28, 0x00, 0x00, 0x00, // header length = 40
        (CHECKSUM & 0xff) as u8,
        ((CHECKSUM >> 8) & 0xff) as u8,
        ((CHECKSUM >> 16) & 0xff) as u8,
        ((CHECKSUM >> 24) & 0xff) as u8,
        0x01, 0x00, 0x00, 0x00, // information request tag type + flags
        0x0c, 0x00, 0x00, 0x00, // information request tag size = 12
        0x06, 0x00, 0x00, 0x00, // request memory map tag
        0x00, 0x00, 0x00, 0x00, // 8-byte alignment padding
        0x00, 0x00, 0x00, 0x00, // end tag type + flags
        0x08, 0x00, 0x00, 0x00, // end tag size = 8
    ],
};
