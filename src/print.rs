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

/// Prints an info message; with a newline
#[macro_export]
macro_rules! info {
    ($string:expr) => {
        let timestamp = $crate::time::time_manager().uptime();

        $crate::print::_print(format_args_nl!(
          concat!("[  {:>3}.{:06}] ", $string),
          timestamp.as_secs(),
          timestamp.subsec_micros(),
        ))
    };
    ($format_string:expr, $($arg:tt)*) => ({
        let timestamp = $crate::time::time_manager().uptime();

        $crate::print::_print(format_args_nl!(
          concat!("[  {:>3}.{:06}] ", $format_string),
          timestamp.as_secs(),
          timestamp.subsec_micros(),
          $($arg)*
        ))
    })
}

/// Prints a warning message; with a newline
#[macro_export]
macro_rules! warn {
    ($string:expr) => {
        let timestamp = $crate::time::time_manager().uptime();

        $crate::print::_print(format_args_nl!(
          concat!("[  {:>3}.{:06}] ", $string),
          timestamp.as_secs(),
          timestamp.subsec_micros(),
        ))
    };
    ($format_string:expr, $($arg:tt)*) => ({
        let timestamp = $crate::time::time_manager().uptime();

        $crate::print::_print(format_args_nl!(
          concat!("[  {:>3}.{:06}] ", $format_string),
          timestamp.as_secs(),
          timestamp.subsec_micros(),
          $($arg)*
        ))
    })
}
