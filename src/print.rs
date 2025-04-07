// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Printing functions

use crate::console;
use core::fmt;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
  console::console().write_fmt(args).unwrap();
}

/// Print without a newline
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::print::_print(format_args!($($arg)*));
    };
}

/// Print with a newline
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
      $crate::print::_print(format_args_nl!($($arg)*));
    })
}