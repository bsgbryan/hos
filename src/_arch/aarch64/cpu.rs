// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

use aarch64_cpu::asm;

pub use asm::nop;

#[cfg(feature = "bsp_rpi3")]
#[inline(always)]
pub fn spin_for_cycles(n: usize) {
  for _ in 0..n { asm::nop(); }
}

#[inline(always)]
pub fn wait_forever() -> ! {
  loop { asm::wfe() }
}