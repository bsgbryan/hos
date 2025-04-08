// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Top-level BCM driver

mod bcm2xxx_gpio;
mod bcm2xxx_pl011_uart;

pub use bcm2xxx_gpio::*;
pub use bcm2xxx_pl011_uart::*;