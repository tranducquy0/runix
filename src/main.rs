//! runix kernel entry point.
//!
//! Boot flow (GRUB → Multiboot2 → this crate):
//! 1. `asm/boot.asm` builds a stack, forwards `eax`/`ebx` to [`kernel_main`].
//! 2. [`kernel_main`] clears the VGA screen, validates the Multiboot2 magic,
//!    and parses the boot-info block for a memory map.
//! 3. Occupied spans (kernel image, boot-info block, boot stack) are reserved
//!    out of the map, a [`FrameAllocator`] is built from the remainder, and a
//!    small alloc/free smoke test exercises the bitmap.
//! 4. Everything halts in `loop {}` — no interrupts or scheduling yet.
//!
//! The Multiboot2 header static itself lives at the bottom of this file in
//! `.multiboot2` so GRUB can find it within the image's first 32 KiB.

// disable normal features that can't build a kernel
#![no_main]
#![no_std]

use core::arch::global_asm;

// driver and handler module
#[macro_use] // use the macros from vga module (KEEP IT AS IS, don't ask why!)
mod drivers;

// core subsystems
mod subs;
use subs::boot::multiboot2::{
    self, BootInformation, Multiboot2Header, CHECKSUM, MULTIBOOT2_ARCHITECTURE_I386,
    MULTIBOOT2_HEADER_LENGTH, MULTIBOOT2_HEADER_MAGIC,
};
use subs::mem::{BootMemoryMap, FrameAllocator, ReservedRange};

// Keep the boot stub in assembly so the exact register/stack handoff from GRUB
// stays explicit and easy to compare against the Multiboot2 spec.
global_asm!(
    include_str!("asm/boot.asm")
);

unsafe extern "C" {
    // These linker / assembly symbols describe physical memory ranges that are
    // already occupied before the allocator starts handing out frames.
    static __kernel_start: u8;
    static __kernel_end: u8;
    static boot_stack: u8;
    static boot_stack_top: u8;
}

// Boot entry: validate Multiboot2 handoff, build the frame allocator, smoke-test alloc/free.
#[unsafe(no_mangle)]
extern "C" fn kernel_main(multiboot_magic: u32, multiboot_info_addr: u32) -> ! {
    // Start from a clean slate: without this, whatever GRUB or the firmware
    // left on screen (menu text, countdown characters) bleeds into the first
    // line of kernel output.
    drivers::output::vga::WRITER.lock().clear_screen();

    println!("Welcome to the R-Unix kernel");

    if multiboot_magic != multiboot2::BOOTLOADER_MAGIC {
        println!("invalid multiboot2 magic: {:#x}", multiboot_magic);
        loop {}
    }

    // `multiboot_info_addr` comes directly from GRUB via `ebx`.
    let grub_boot_info = unsafe { BootInformation::load(multiboot_info_addr as usize) };
    let Some(mem_map_tag) = grub_boot_info.memory_map_tag() else {
        println!("GRUB did not provide a memory map tag");
        loop {}
    };

    // Copy the bootloader's raw memory description into a small kernel-owned
    // structure before we start carving reserved ranges out of it.
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

    // The allocator is fallible now: it refuses to track more frames than its
    // fixed bitmap can hold, so handle that loudly instead of panicking.
    let mut allocator = match FrameAllocator::new(&memory_map) {
        Some(allocator) => allocator,
        None => {
            println!("boot memory map exceeds allocator tracking capacity");
            loop {}
        }
    };

    println!(
        "frame allocator ready: {} total, {} free",
        allocator.total_frames(),
        allocator.free_frames()
    );

    // Smoke test: allocate a few frames, free one, and reallocate to prove the
    // bitmap tracks frees and reuses frames instead of only bumping forward.
    println!("allocating a few frames:");
    let mut frames = [0usize; 4];
    let mut allocated = 0usize;
    for slot in &mut frames {
        match allocator.alloc() {
            Some(frame) => {
                *slot = frame;
                allocated += 1;
                println!("  frame = {:#x}", frame);
            }
            None => {
                println!("  no more frames");
                break;
            }
        }
    }

    if allocated > 0 {
        println!("freeing first allocated frame: {:#x}", frames[0]);
        if !allocator.free(frames[0]) {
            println!("  free failed");
        } else {
            match allocator.alloc() {
                Some(frame) => println!("  reused after free: {:#x}", frame),
                None => println!("  no frame reused after free"),
            }
        }
    }

    println!(
        "frame allocator stats: {} total, {} used, {} free",
        allocator.total_frames(),
        allocator.used_frames(),
        allocator.free_frames()
    );

    loop {}
}

#[unsafe(link_section = ".multiboot2")]
#[used]
#[unsafe(no_mangle)]
static MULTIBOOT2_HEADER: Multiboot2Header = Multiboot2Header {
    data: [
        // Standard Multiboot2 header prologue: magic, architecture, size,
        // then a checksum that makes the 32-bit sum of these fields equal zero.
        MULTIBOOT2_HEADER_MAGIC.to_le_bytes()[0],
        MULTIBOOT2_HEADER_MAGIC.to_le_bytes()[1],
        MULTIBOOT2_HEADER_MAGIC.to_le_bytes()[2],
        MULTIBOOT2_HEADER_MAGIC.to_le_bytes()[3],
        MULTIBOOT2_ARCHITECTURE_I386.to_le_bytes()[0],
        MULTIBOOT2_ARCHITECTURE_I386.to_le_bytes()[1],
        MULTIBOOT2_ARCHITECTURE_I386.to_le_bytes()[2],
        MULTIBOOT2_ARCHITECTURE_I386.to_le_bytes()[3],
        MULTIBOOT2_HEADER_LENGTH.to_le_bytes()[0],
        MULTIBOOT2_HEADER_LENGTH.to_le_bytes()[1],
        MULTIBOOT2_HEADER_LENGTH.to_le_bytes()[2],
        MULTIBOOT2_HEADER_LENGTH.to_le_bytes()[3],
        (CHECKSUM & 0xff) as u8,
        ((CHECKSUM >> 8) & 0xff) as u8,
        ((CHECKSUM >> 16) & 0xff) as u8,
        ((CHECKSUM >> 24) & 0xff) as u8,
        // Request that GRUB include the memory map boot info tag.
        0x01, 0x00, 0x00, 0x00, // information request tag type + flags
        0x0c, 0x00, 0x00, 0x00, // information request tag size = 12
        0x06, 0x00, 0x00, 0x00, // request memory map tag
        0x00, 0x00, 0x00, 0x00, // 8-byte alignment padding
        // Required terminating tag for Multiboot2 header parsing.
        0x00, 0x00, 0x00, 0x00, // end tag type + flags
        0x08, 0x00, 0x00, 0x00, // end tag size = 8
    ],
};
