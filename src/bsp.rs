// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Conditional reexporting of Board Support Packages.

#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
mod raspberrypi;

/*
  Removed this because in edition 2024 this generates a warning,
  which causes compilation to fail - and commenting it out doesn't
  seem to break anything
*/
// #[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
// pub use raspberrypi::*;
