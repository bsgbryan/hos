// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//--------------------------------------------------------------------------------------------------
// Definitions
//--------------------------------------------------------------------------------------------------
.macro ADR_REL register, symbol
	adrp \register, \symbol
	add	 \register, \register, #:lo12:\symbol
.endm

//--------------------------------------------------------------------------------------------------
// Resources
//--------------------------------------------------------------------------------------------------

// Load the address of a symbol into a register, absolute
// See: https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
// .macro ADR_ABS register, symbol
// 	movz \register, #:abs_g2:\symbol
// 	movk \register, #:abs_g1_nc:\symbol
// 	movk \register, #:abs_g0_nc:\symbol
// .endm

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
.section .text._start

//------------------------------------------------------------------------------
// fn _start()
//------------------------------------------------------------------------------
_start:
	// Only proceed if the core is in EL2, park it otherwise
	mrs x0, CurrentEL
	cmp x0, {CONST_CURRENTEL_EL2}
	b.ne .L_parking_loop
	// Only proceed on the boot core (for now). Park it otherwise
	mrs x1, MPIDR_EL1
	and x1, x1, {CONST_CORE_ID_MASK}
	ldr x2, BOOT_CORE_ID // provided by bsp/__board_name__/cpu.rs
	cmp x1, x2
	b.ne .L_parking_loop
	// If execution reaches here, it is the boot core
	// Initialize DRAM
	ADR_REL x0, __bss_start
	ADR_REL x1, __bss_end_exclusive

.L_bss_init_loop:
	cmp x0, x1
	b.eq .L_prepare_rust
	stp xzr, xzr, [x0], #16
	b .L_bss_init_loop
// .L_relocate_binary:
// 	ADR_REL x0, __binary_nonzero_start // The address the binary was loaded into
// 	ADR_ABS x1, __binary_nonzero_start // The address the binary was linked to
// 	ADR_ABS x2, __binary_nonzero_end_exclusive
// .L_copy_loop:
// 	ldr x3, [x0], #8
// 	str x3, [x1], #8
// 	cmp x1, x2
// 	b.lo .L_copy_loop
// Prepare the jump to Rust code
.L_prepare_rust:
	// Set the stack pointer
	// This ensures that any code in EL2 that needs the stack will work
	ADR_REL x0, __boot_core_stack_end_exclusive
	mov sp, x0
	// Read the CPU's timer counter frequency and store it in ARCH_TIMER_COUNTER_FREQUENCY
	// Abort if the frequency read back is 0
	ADR_REL x1, ARCH_TIMER_COUNTER_FREQUENCY // Provided by aarch64/time.rs
	mrs x2, CNTFRQ_EL0
	cmp x2, xzr
	b.eq .L_parking_loop
	str w2, [x1]
	// Jump to Rust code
	// x0 holds the function argument provided to `_start_rust`
	b _start_rust
// Infinitely wait for events (aka "park the core")
.L_parking_loop:
	wfe
	b	.L_parking_loop

.size	_start, . - _start
.type	_start, function
.global	_start
