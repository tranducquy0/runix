//! Task State Segment (TSS): the descriptor-table sibling the boot path never
//! built for us.
//!
//! Two TSSes live here:
//! - the kernel's own (selector `0x18` in [`super::gdt`]) — `esp0`/`ss0` name
//!   the stack the CPU would switch onto for a privilege change (we run ring
//!   0 only, so nothing switches yet), and the same table becomes the place
//!   hardware stores the interrupted context when a task-gate fires;
//! - the double-fault task in [`crate::subs::handlers::doublefault`], which
//!   shares this layout but gets its own static table and stack.
//!
//! Layout per the SDM 32-bit TSS: every `esp`/`ss` register pair is padded to
//! a dword boundary, the general-purpose block sits at fixed offsets, segment
//! snapshots are 16-bit each, and the I/O-map base is the final word at
//! 0x66. The debug-trap `T` byte inside the reserved tail stays zero so task
//! switches never raise #DB.

/// Fixed total size of a 32-bit TSS (SDM: the table is 104 bytes).
pub const TSS_SIZE: u16 = 0x68;

/// A base past the end of the table: encodes "no I/O permission bitmap," so
/// every `in`/`out` with IOPL < 3 faults instead of silently succeeding
/// against a fabricated permission map.
pub const IOMAP_NONE: u16 = TSS_SIZE;

/// The 104-byte 32-bit Task State Segment, packed exactly as the CPU decodes
/// it. `#[repr(packed)]` because the hardware layout is byte-precise — the
/// mixed 16/32-bit fields leave no room for default alignment. Field names
/// mirror the SDM; the `_reserved_*` words are the half-dword pads the CPU
/// never touches.
#[repr(C, packed)]
pub struct Tss {
    /// Back link, kept by hardware on nested task switches.
    pub link: u16,
    _reserved_link: u16,    /// Stack pointer the CPU loads on a privilege change into ring 0.
    pub esp0: u32,
    /// Stack segment paired with the `esp0` stack (kernel data: 0x10).
    pub ss0: u16,
    _reserved_ss0: u16,
    pub esp1: u32,
    pub ss1: u16,
    _reserved_ss1: u16,
    pub esp2: u32,
    pub ss2: u16,
    _reserved_ss2: u16,
    /// Page-table base, loaded by the hardware task switch. Zero while
    /// paging is off.
    pub cr3: u32,
    /// Instruction pointer the CPU jumps to when switching INTO this task.
    pub eip: u32,
    pub eflags: u32,
    pub eax: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebx: u32,
    pub esp: u32,
    pub ebp: u32,
    pub esi: u32,
    pub edi: u32,
    // Unlike the x86_64 TSS, the 32-bit TSS gives every segment snapshot
    // its own dword row (selector in the low word, reserved upper half) —
    // verified against real delivery behavior via QEMU `-d int`: with a
    // contiguous (x86_64-style) segment block the CPU loaded CS from the
    // 0x4C slot and immediately triple-faulted on a non-executable CS.
    pub es: u16,
    _reserved_es: u16,
    pub cs: u16,
    _reserved_cs: u16,
    pub ss: u16,
    _reserved_ss: u16,
    pub ds: u16,
    _reserved_ds: u16,
    pub fs: u16,
    _reserved_fs: u16,
    pub gs: u16,
    _reserved_gs: u16,
    pub ldt: u16,
    // 0x62..0x66: reserved, including the debug-trap flag at 0x64 we want
    // to hold at zero so entering a task never raises #DB.
    _reserved_tail: [u8; 4],
    /// Offset (from the TSS base) of the I/O permission bitmap.
    pub io_bitmap_base: u16,
}

impl Tss {
    /// An all-zero table whose only non-zero value is `io_bitmap_base =
    /// IOMAP_NONE` — the "no I/O bitmap" state that ring 0 tolerates. Shared
    /// starting point for both static TSSes.
    pub const fn zeroed() -> Self {
        Self {
            link: 0,
            _reserved_link: 0,
            esp0: 0,
            ss0: 0,
            _reserved_ss0: 0,
            esp1: 0,
            ss1: 0,
            _reserved_ss1: 0,
            esp2: 0,
            ss2: 0,
            _reserved_ss2: 0,
            cr3: 0,
            eip: 0,
            eflags: 0,
            eax: 0,
            ecx: 0,
            edx: 0,
            ebx: 0,
            esp: 0,
            ebp: 0,
            esi: 0,
            edi: 0,
            es: 0,
            _reserved_es: 0,
            cs: 0,
            _reserved_cs: 0,
            ss: 0,
            _reserved_ss: 0,
            ds: 0,
            _reserved_ds: 0,
            fs: 0,
            _reserved_fs: 0,
            gs: 0,
            _reserved_gs: 0,
            ldt: 0,
            _reserved_tail: [0; 4],
            io_bitmap_base: IOMAP_NONE,
        }
    }
}

#[used]
static mut TSS: Tss = Tss::zeroed();

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

/// EIP hardware saved in this table when a task gate switched out of it —
/// i.e. the faulting instruction for the double-fault path. Meaningless
/// before any such switch; the table only ever gets written by hardware once.
pub fn saved_eip() -> u32 {
    let tss = &raw const TSS;

    unsafe { (*tss).eip }
}

/// ESP hardware saved alongside [`saved_eip`].
pub fn saved_esp() -> u32 {
    let tss = &raw const TSS;

    unsafe { (*tss).esp }
}
