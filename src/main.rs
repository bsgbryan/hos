// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//!
//! HOS' kernel
//! 

#![feature(format_args_nl)]
#![no_main]
#![no_std]

mod bsp;
mod console;
mod cpu;
mod panic_wait;
mod print;

///
/// Initialize the kernel
/// 
unsafe fn kernel_init() -> ! {
  println!("Hello from Rust!");
  panic!("Stopping here")
}