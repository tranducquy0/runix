//! Raw 40-byte Multiboot2 image header plus the constants it is built from.
//!
//! Everything here is what *GRUB reads from our image*: the header static is
//! placed in `.multiboot2` by the linker ... GRUB scans the image's first 32
//! KiB for it. The inverse data flow — reading the boot-info block GRUB hands
//! back — lives in the sibling [`super::boot_info`] module.

/// Multiboot2 image-header magic.
pub const MULTIBOOT2_HEADER_MAGIC: u32 = 0xE85250D6;
pub const MULTIBOOT2_ARCHITECTURE_I386: u32 = 0;
pub const MULTIBOOT2_HEADER_LENGTH: u32 = 40;

// Multiboot2 header constants: 32-bit sum of magic/arch/len/checksum must be zero.
pub const CHECKSUM: u32 = (0u32).wrapping_sub(
    MULTIBOOT2_HEADER_MAGIC + MULTIBOOT2_ARCHITECTURE_I386 + MULTIBOOT2_HEADER_LENGTH,
);

/// Raw 40-byte Multiboot2 header placed in `.multiboot2` by the linker.
///
/// GRUB scans the image's first 32 KiB for this structure; its byte array
/// encodes magic + architecture + length + checksum, an information-request
/// tag asking for the memory map (tag 6), and the required end tag. All
/// fields are literal bytes so GRUB sees exactly the on-disk layout the
/// spec demands, independent of Rust layout rules.
#[repr(C, align(8))]
pub struct Multiboot2Header {
    // Raw bytes avoid Rust layout surprises in a format where GRUB expects
    // exact offsets and exact alignment.
    pub data: [u8; MULTIBOOT2_HEADER_LENGTH as usize],
}
