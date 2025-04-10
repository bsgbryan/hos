// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Architectural boot code.
//!
//! # Overview
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::boot::arch_boot

use core::arch::global_asm;

use aarch64_cpu::{
  asm,
  registers::*,
};
use tock_registers::interfaces::Writeable;

// Assembly counterpart to this file.
global_asm!(
  include_str!("boot.s"),
  CONST_CURRENTEL_EL2 = const 0x8,
  CONST_CORE_ID_MASK = const 0b11,
);

/// The Rust entry point for the `kernal` binary
/// 
/// This is called from the assembly `_start` function
/// 
/// # Safety
/// 
/// - Exception return from EL2 must continue excution in EL1 with `kernel_init`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
  unsafe { prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr); }

  // Use `eret` to "return" to EL1
  // This results in execution of `kernel_init` in EL1
  asm::eret()
}

/// Prepares the transition EL2 -> EL1
/// 
/// # Safety
/// 
/// - The `bss` section is not initialized yet; the code must not use or reference it in any way
/// - The hardware state of EL1 must be prepared in a sound way
#[inline(always)]
unsafe fn prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr: u64) {
  // Enable timer counter registers for EL1
  CNTHCTL_EL2.write(
    CNTHCTL_EL2::EL1PCEN::SET
    +
    CNTHCTL_EL2::EL1PCTEN::SET
  );

  // No offset for reading the counters
  CNTVOFF_EL2.set(0);

  // Set EL1 execution state to AArch64
  HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

  // Setup a simulated exception return
  // 1. Fake a saved program status where all interrupts were masked and SP_EL1 was used as a stack pointer
  SPSR_EL2.write(
    SPSR_EL2::D::Masked +
    SPSR_EL2::A::Masked +
    SPSR_EL2::I::Masked +
    SPSR_EL2::F::Masked +
    SPSR_EL2::M::EL1h,
  );

  // 2. Let the link register point to `kernel_init`
  ELR_EL2.set(crate::kernel_init as *const () as u64);

  // 3. Setup SP_EL1 (stack pointer) - which will be used by EL1 once we "return" to it
  //    Since there are no plans to ever return to EL2 we use re-use EL2's stack for EL1
  SP_EL1.set(phys_boot_core_stack_end_exclusive_addr);
}