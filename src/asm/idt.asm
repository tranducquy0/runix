# Minimal IDT setup for runix
#
# Loads the kernel's IDT and provides entry stubs for CPU exceptions.
# Exception stubs save registers, call the Rust handler, restore state,
# and return with iret. For exceptions without error codes pushed by
# the CPU, we push a dummy error code to keep the stack layout uniform.
#
# extern "C" fn idt_load(idt_pointer_addr: u32)

.section .text

# Load IDT pointer and return
.global idt_load
idt_load:
    mov eax, [esp + 4]
    lidt [eax]
    ret

# Define stubs for exceptions 0-31
.global isr0
isr0:
    cli
    push 0
    push 0
    jmp isr_common_stub

.global isr1
isr1:
    cli
    push 0
    push 1
    jmp isr_common_stub

.global isr2
isr2:
    cli
    push 0
    push 2
    jmp isr_common_stub

.global isr3
isr3:
    cli
    push 0
    push 3
    jmp isr_common_stub

.global isr4
isr4:
    cli
    push 0
    push 4
    jmp isr_common_stub

.global isr5
isr5:
    cli
    push 0
    push 5
    jmp isr_common_stub

.global isr6
isr6:
    cli
    push 0
    push 6
    jmp isr_common_stub

.global isr7
isr7:
    cli
    push 0
    push 7
    jmp isr_common_stub

.global isr8
isr8:
    cli
    push 8
    jmp isr_common_stub

.global isr9
isr9:
    cli
    push 0
    push 9
    jmp isr_common_stub

.global isr10
isr10:
    cli
    push 10
    jmp isr_common_stub

.global isr11
isr11:
    cli
    push 11
    jmp isr_common_stub

.global isr12
isr12:
    cli
    push 12
    jmp isr_common_stub

.global isr13
isr13:
    cli
    push 13
    jmp isr_common_stub

.global isr14
isr14:
    cli
    push 14
    jmp isr_common_stub

.global isr15
isr15:
    cli
    push 0
    push 15
    jmp isr_common_stub

.global isr16
isr16:
    cli
    push 0
    push 16
    jmp isr_common_stub

.global isr17
isr17:
    cli
    push 17
    jmp isr_common_stub

.global isr18
isr18:
    cli
    push 0
    push 18
    jmp isr_common_stub

.global isr19
isr19:
    cli
    push 0
    push 19
    jmp isr_common_stub

.global isr20
isr20:
    cli
    push 0
    push 20
    jmp isr_common_stub

.global isr21
isr21:
    cli
    push 0
    push 21
    jmp isr_common_stub

.global isr22
isr22:
    cli
    push 0
    push 22
    jmp isr_common_stub

.global isr23
isr23:
    cli
    push 0
    push 23
    jmp isr_common_stub

.global isr24
isr24:
    cli
    push 0
    push 24
    jmp isr_common_stub

.global isr25
isr25:
    cli
    push 0
    push 25
    jmp isr_common_stub

.global isr26
isr26:
    cli
    push 0
    push 26
    jmp isr_common_stub

.global isr27
isr27:
    cli
    push 0
    push 27
    jmp isr_common_stub

.global isr28
isr28:
    cli
    push 0
    push 28
    jmp isr_common_stub

.global isr29
isr29:
    cli
    push 0
    push 29
    jmp isr_common_stub

.global isr30
isr30:
    cli
    push 30
    jmp isr_common_stub

.global isr31
isr31:
    cli
    push 0
    push 31
    jmp isr_common_stub

# Common stub that saves state and calls Rust handler
isr_common_stub:
    pusha
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    push esp
    call isr_handler
    add esp, 4
    popa
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    add esp, 8
    iret
