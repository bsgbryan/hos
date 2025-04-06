// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! System console

use crate::bsp;

pub mod interface {
  pub use core::fmt::Write;
}

pub fn console() -> impl interface::Write {
  bsp::console::console()
}