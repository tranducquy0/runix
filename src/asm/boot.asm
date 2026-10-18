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
