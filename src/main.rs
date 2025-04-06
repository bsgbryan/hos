// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//!
//! HOS' kernel
//! 

#![no_main]
#![no_std]

mod bsp;
mod cpu;
mod panic_wait;

///
/// Initialize the kernel
/// 
unsafe fn kernel_init() -> ! {
  panic!()
}