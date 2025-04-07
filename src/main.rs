// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//!
//! HOS' kernel
//! 

#![feature(format_args_nl)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

mod bsp;
mod console;
mod cpu;
mod panic_wait;
mod print;
mod synchronization;

///
/// Initialize the kernel
/// 
unsafe fn kernel_init() -> ! {
  use console::console;

  println!("[0] Hello from Rust!");
  println!("[1] Chars written: {}", console().char_written());
  println!("[2] Stopping here");

  cpu::wait_forever()
}