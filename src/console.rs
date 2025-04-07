// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! System console

use crate::bsp;

pub mod interface {
  use core::fmt;

  pub trait Write {
    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
  }

  pub trait Statistics {
    fn char_written(&self) -> usize { 0 }
  }

  pub trait All: Write + Statistics {}
}

pub fn console() -> &'static dyn interface::All {
  bsp::console::console()
}