// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! System console

mod null_console;

use crate::synchronization::{
  NullLock,
  self,
};

pub mod interface {
  use core::fmt;

  /// Console write functions
  pub trait Write {
    /// Write a single character
    fn write_char(&self, c: char);

    /// Write a Rust format string
    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;

    // Block until the last buffered character has been physically put on the TX wire
    // fn flush(&self);
  }

  /// Console read functions
  pub trait Read {
    /// Read a single character
    fn read_char(&self) -> char {
      ' '
    }

    /// Clear the RX buffer
    fn clear_rx(&self);
  }

  /// Super-fun console statistics!
  pub trait Statistics {
    // The number of characters written
    // fn chars_written(&self) -> usize { 0 }

    // Why is this here if it isn't used?
    // This caused hours of frustration because rustc was throwing dead code warning - which this project treats like errors
    // The number of character read
    // fn chars_read(&self) -> usize { 0 }
  }

  pub trait All: Write + Read + Statistics {}
}

static CUR_CONSOLE: NullLock<&'static (dyn interface::All + Sync)> = NullLock::new(&null_console::NULL_CONSOLE);

use synchronization::interface::Mutex;

/// Register the console
pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
  CUR_CONSOLE.lock(|c| *c = new_console);
}

pub fn console() -> &'static dyn interface::All {
  CUR_CONSOLE.lock(|c| *c)
}