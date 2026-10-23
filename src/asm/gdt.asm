# One-shot GDT install stub.
#
# GRUB hands us protected mode on its own GDT; this stub repoints GDTR at the
# kernel's table (see subs/arch/gdt.rs) and reloads every segment register
# back-to-back, so no instruction runs with a mix of GRUB's and runix's
# selectors. The hard-coded selector values below must stay in sync with
# KERNEL_CODE_SELECTOR / KERNEL_DATA_SELECTOR in subs/arch/gdt.rs.
#
# extern "C" fn gdt_load_flush(gdt_pointer_addr: u32)
# extern "C" fn tss_load(tss_selector: u32)
.section .text
.global gdt_load_flush
gdt_load_flush:
    # The lgdt operand is a 6-byte limit/base pair living behind the C-ABI
    # pointer argument; `lgdt` copies it into GDTR and retires immediately,
    # so a stack-local pointer is fine.
    mov eax, [esp + 4]
    lgdt [eax]

    # Reload every data segment register onto the flat ring-0 data selector.
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    # Reload CS with a far return: the selector is pushed first (higher slot)
    # and the resume address last, so `retf` pops EIP then the selector and
    # execution continues right after the far return — now running on the
    # kernel's own code segment.
    push dword ptr 0x08
    mov eax, offset .resume_in_new_code
    push eax
    retf

.resume_in_new_code:
    ret

# The companion TR load: `ltr` needs a plain u16 selector, which the C ABI
# passes in the low half of EAX. Runs immediately after `gdt_load_flush`,
# while the new GDT is already live. No flush needed — TR metadata updates
# lazily on `ltr` itself.
#
# extern "C" fn tss_load(tss_selector: u32)
.global tss_load
tss_load:
    # Selector sits right behind the return address in EAX's low half.
    mov ax, [esp + 4]
    ltr ax
    ret
