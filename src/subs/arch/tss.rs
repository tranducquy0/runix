//! Task State Segment (TSS): the descriptor-table sibling the boot path never
//! built for us.
//!
//! One available-32-bit TSS, registered in the kernel's own GDT at selector
//! `0x18` (see [`super::gdt`]). This step is deliberate groundwork, not a
//! feature: `esp0`/`ss0` name the stack the CPU would switch onto for a
//! privilege change (we run ring 0 only, so nothing switches today), and the
//! planned double-fault stack handling routes through the same table.
//!
//! Layout per the SDM 32-bit TSS: `link` at 0x00, `esp0` at 0x04, `ss0` at
//! 0x08, and the I/O-map base as the final word at 0x66. The middle is
//! hardware task-switch state (CR3, saved registers, segment snapshots) this
//! kernel will never machine-switch — it stays reserved rather than being
//! modelled with fields we don't use.

/// Fixed total size of a 32-bit TSS (SDM: the table is 104 bytes).
const TSS_SIZE: u16 = 0x68;

/// Byte offset of the I/O-map base field (the last word of the table).
const OFFSET_IO_BITMAP_BASE: u16 = 0x66;

/// Reserved span between `ss0` (ends at 0x0A) and the I/O-map base.
/// Field offsets up to here (`link` 0x00, `esp0` 0x04, `ss0` 0x08) are the
/// contract we rely on; the rest is hardware task-switch state.
const RESERVED_SPAN: usize = (OFFSET_IO_BITMAP_BASE - 0x0A) as usize; // 0x5C

/// The 104-byte 32-bit Task State Segment, packed exactly as the CPU reads
/// it. `#[repr(packed)]` because the hardware layout is byte-precise — the
/// mixed 16/32-bit fields leave no room for default alignment.
#[repr(C, packed)]
pub struct Tss {
    /// Back link. Only meaningful for hardware task chains; stays 0.
    pub link: u32,
    /// Stack pointer the CPU loads on a privilege change into ring 0.
    pub esp0: u32,
    /// Stack segment paired with the `esp0` stack (kernel data: 0x10).
    pub ss0: u16,
    reserved: [u8; RESERVED_SPAN],
    /// Offset (from the TSS base) of the I/O permission bitmap. A base past
    /// the table end means "no I/O bitmap", which is what we want while ring
    /// 0 is the only privilege level.
    pub io_bitmap_base: u16,
}

#[used]
static mut TSS: Tss = Tss {
    link: 0,
    esp0: 0,
    ss0: 0,
    reserved: [0; RESERVED_SPAN],
    // No I/O bitmap: insert nothing, every `in`/`out` fault instead of
    // silently succeeding against a fabricated permission map.
    io_bitmap_base: TSS_SIZE,
};

/// Linear address of the static TSS table, for the GDT descriptor builder
/// to point entry selector `0x18` at.
pub fn address() -> u32 {
    &raw const TSS as u32
}

/// Point the TSS's privilege-change stack at the top of `esp0`, paired with
/// data selector `ss0`. Called from `kernel_main` right after the GDT lands,
/// so nothing can run with an empty esp0/ss0 pair by accident.
pub fn set_kernel_stack(esp0: u32, ss0: u16) {
    let tss = &raw mut TSS;
    unsafe {
        (*tss).esp0 = esp0;
        (*tss).ss0 = ss0;
    }
}
