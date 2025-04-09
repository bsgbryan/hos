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

const MINILOAD_LOGO: &str = r#"
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|
"#;

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

  println!("{}", MINILOAD_LOGO);
  println!("{:^37}", bsp::board_name());
  println!();
  println!("[ML] Requesting binary");
  console().flush();

  // Discard any spurious characters received before going into echo mode
  console().clear_rx();

  // Notify `Minipush` to send the binary
  for _ in 0..3 { console().write_char(3 as char); }

  // Read the binary's size
  let mut size: u32 = u32::from(console().read_char() as u8);
  size |= u32::from(console().read_char() as u8) <<  8;
  size |= u32::from(console().read_char() as u8) << 16;
  size |= u32::from(console().read_char() as u8) << 24;

  // Trust it's not too big
  console().write_char('O');
  console().write_char('K');

  let kernel_addr: *mut u8 = bsp::memory::board_default_load_addr() as *mut u8;
  unsafe {
    // Read the kernel byte by byte
    for i in 0..size {
      core::ptr::write_volatile(
        kernel_addr.offset(i as isize),
        console().read_char() as u8
      )
    }
  }

  println!("[ML] Loaded! Executing the payload now\n");
  console().flush();

  // Use dark magic to create a function pointer
  let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };

  // Jump to the loaded kernel!
  kernel()
}