// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! BSP console utilities

use crate::console;
use core::fmt;

struct QEMUOutput;

impl fmt::Write for QEMUOutput {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    for c in s.chars() {
      unsafe {
        core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
      }
    }

    Ok(())
  }
}

pub fn console() -> impl console::interface::Write {
  QEMUOutput {}
}
