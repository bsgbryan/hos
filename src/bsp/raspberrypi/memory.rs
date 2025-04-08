// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! RPi memory management

// The board's physical memory map
pub(super) mod map {
  pub const GPIO_OFFSET: usize = 0x0020_0000;
  pub const UART_OFFSET: usize = 0x0020_1000;

  /// Physical devices
  #[cfg(feature = "bsp_rpi3")]
  pub mod mmio {
    use super::*;

    pub const START:            usize = 0x3F00_0000;
    pub const GPIO_START:       usize = START + GPIO_OFFSET;
    pub const PL011_UART_START: usize = START + UART_OFFSET;
  }

  /// Physical devices
  #[cfg(feature = "bsp_rpi4")]
  pub mod mmio {
    use super::*;

    pub const START:            usize = 0xFE00_0000;
    pub const GPIO_START:       usize = START + GPIO_OFFSET;
    pub const PL011_UART_START: usize = START + UART_OFFSET;
  }
}