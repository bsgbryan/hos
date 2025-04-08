// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//!
//! HOS' kernel
//! 

#![allow(clippy::upper_case_acronyms)]
#![feature(format_args_nl)]
#![feature(sync_unsafe_cell)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

mod bsp;
mod console;
mod cpu;
mod driver;
mod panic_wait;
mod print;
mod synchronization;

///
/// Initialize the kernel
/// 
fn kernel_init() -> ! {
  // Initialize the BSP driver subsystem
  if let Err(e) = unsafe { bsp::driver::init() } {
    panic!("Error initializing BSP driver subsystem: {}", e);
  }

  // Initialize all device drivers
  driver::driver_manager().init_drivers();
  // println! is usable from here on

  // Transition unsafe -> safe
  kernel_main()
}

/// main kernel function
fn kernel_main() -> ! {
  use console::console;

  println!("{} version {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
  println!("Booting on: {}", bsp::board_name());
  println!("Drivers loaded:");
  driver::driver_manager().enumerate();

  println!("Chars written: {}", console().chars_written());
  println!("Echoing input now");

  // Discard any spurious characters received before going into echo mode
  console().clear_rx();

  loop {
    let c = console().read_char();
    console().write_char(c);
  }
}