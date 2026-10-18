# Multiboot2 entry stub.
#
# GRUB jumps to `_start` in 32-bit protected mode with:
#   eax = Multiboot2 magic (checked later in Rust)
#   ebx = physical address of the boot-info block
# No stack, no frame, no red zone — this file must build the stack, move
# those registers into C-ABI arguments (cdecl: push right-to-left), and
# enter `kernel_main`. Returning from `kernel_main` is unsupported.

.section .bss
    .align 16
    # Export the temporary stack range so Rust can mark it reserved before any
    # physical frames are handed out.
    .global boot_stack
    .global boot_stack_top
boot_stack:
    .skip 32768
boot_stack_top:

    .section .text
    .global _start
_start:
    # GRUB arrives here with Multiboot2 values in eax/ebx but without a Rust ABI
    # stack frame. Build a stack and forward those values as C arguments.
    mov esp, offset boot_stack_top
    push ebx
    push eax
    call kernel_main
1:
    # Returning from `kernel_main` is not supported yet, so stop forever.
    hlt
    jmp 1b
