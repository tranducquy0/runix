//! Layout contract of the 32-bit TSS (`src/subs/arch/tss.rs`).
//!
//! `ltr`, esp0/ss0 pairing and the no-bitmap encoding are hardware behavior
//! covered by the live QEMU boot check; here we pin the byte layout the CPU
//! decodes, so it can't drift when the module grows.

use core::mem::{offset_of, size_of};
use runix_tests::subs::arch::tss::Tss;

// The SDM fixes a 32-bit TSS at 104 bytes, with the I/O-map base word as the
// final field at the table's last two bytes.
#[test]
fn table_is_104_bytes_and_io_base_word_is_last() {
    assert_eq!(size_of::<Tss>(), 104);
    assert_eq!(offset_of!(Tss, io_bitmap_base), size_of::<Tss>() - 2);
}

// esp0 at 0x04 and ss0 (a u16) at 0x08: the CPU loads exactly these slots on
// a privilege switch, so a shift here would silently redirect that stack.
#[test]
fn privilege_change_stack_lands_at_the_cpu_offsets() {
    assert_eq!(offset_of!(Tss, esp0), 0x04);
    assert_eq!(offset_of!(Tss, ss0), 0x08);
}

// Back link is the first field, as the SDM layout demands.
#[test]
fn link_field_first() {
    assert_eq!(offset_of!(Tss, link), 0x00);
}
