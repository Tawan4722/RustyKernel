OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

SECTIONS
{
    . = 0xffffffff80000000;

    .text : {
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .limine_req_start : {
        KEEP(*(.limine_req_start))
    }

    .limine_reqs : {
        KEEP(*(.limine_reqs))
    }

    .limine_req_end : {
        KEEP(*(.limine_req_end))
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        *(COMMON)
        *(.bss .bss.*)
    }
}
