ENTRY(_start)

SECTIONS {
    . = 1M;
    .multiboot2 : {
        . = ALIGN(0);
        *(.multiboot2)
   }
    .text : {
        *(.text*)
    }

    .rodata : {
        *(.rodata*)
    }

    .data : {
        *(.data*)
    }

    .bss : {
        *(.bss*)
    }
}