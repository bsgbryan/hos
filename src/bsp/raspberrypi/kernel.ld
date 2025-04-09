/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>
 */

/* The physical address at which the kernel binary will be loaded by the RPi's firmware */
__rpi_phys_binary_load_addr = 0x80000;


ENTRY(__rpi_phys_binary_load_addr)

/* Flags:
 *     4 == R
 *     5 == RX
 *     6 == RW
 *
 * Segments are marked PT_LOAD below so that the ELF file provides virtual and physical addresses.
 * It doesn't mean all of them need to actually be loaded.
 */
PHDRS {
    segment_boot_core_stack PT_LOAD FLAGS(6);
    segment_code            PT_LOAD FLAGS(5);
    segment_data            PT_LOAD FLAGS(6);
}

SECTIONS {
    /* Set the link address to 32MiB */
    . = 0x2000000;

    .boot_core_stack(NOLOAD): {
        . += __rpi_phys_binary_load_addr;
        __boot_core_stack_end_exclusive = .;
    } :segment_boot_core_stack

    __binary_nonzero_start = .;
    .text : {
        KEEP(*(.text._start))
        *(.text._start_arguments)
        *(.text._start_rust)
        *(.text*)
    } :segment_code
    .rodata : ALIGN(8) { *(.rodata*) } :segment_code

    .data : { *(.data*) } :segment_data

    /* Fill up to 8 byte, because relocating the binary is done in u64 chunks */
    . = ALIGN(8);
    __binary_nonzero_end_exclusive = .;

    .bss(NOLOAD): ALIGN(16) {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(16);
        __bss_end_exclusive = .;
    } : segment_data

    .got : { *(.got*) }
    ASSERT(SIZEOF(.got) == 0, "Relocation support not expected")

    /DISCARD/ : { *(.comment*) }
}
