// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! A console that does nothing

use super::interface;
use core::fmt;

pub struct NullConsole;

pub static NULL_CONSOLE: NullConsole = NullConsole {};

impl interface::Write for NullConsole {
  // fn write_char(&self, _c: char) {}

  fn write_fmt(&self, _args: fmt::Arguments) -> fmt::Result {
    fmt::Result::Ok(())
  }

  // fn flush(&self) {}
}

impl interface::Read for NullConsole {
  // fn clear_rx(&self) {}
}

impl interface::Statistics for NullConsole {}
impl interface::All for NullConsole {}

unsafe impl Sync for NullConsole {}