// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Top-level BSP file for the Raspberry Pi 3 and 4.

pub mod cpu;
pub mod driver;
pub mod memory;

/// Board identification
pub fn board_name() -> &'static str {
  #[cfg(feature = "bsp_rpi3")]
  { "Raspberry Pi 3" }

  #[cfg(feature = "bsp_rpi4")]
  { "Raspberry Pi 4" }
}
