.section .bss
    .align 16
    .global boot_stack
    .global boot_stack_top
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
