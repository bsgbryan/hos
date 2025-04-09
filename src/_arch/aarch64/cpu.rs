// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

use aarch64_cpu::asm;

pub use asm::nop;

#[inline(always)]
pub fn wait_forever() -> ! {
  loop { asm::wfe() }
}