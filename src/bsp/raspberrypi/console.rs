// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! BSP console utilities

use crate::console;

pub fn console() -> &'static dyn console::interface::All {
  &super::driver::PL011_UART
}
