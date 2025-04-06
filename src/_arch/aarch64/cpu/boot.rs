// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Architectural boot code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::boot::arch_boot

use core::arch::global_asm;

// Assembly counterpart to this file.
global_asm!(
  include_str!("boot.s"),
  CONST_CORE_ID_MASK = const 0b11,
);

/// The Rust entry point for the `kernal` binary
/// 
/// This is called from the assembly `_start` function
#[unsafe(no_mangle)]
pub fn _start_rust() -> ! {
  unsafe { crate::kernel_init() }
}