// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//!
//! HOS' kernel
//! 

#![allow(clippy::upper_case_acronyms)]
#![allow(incomplete_features)]
#![allow(internal_features)]
#![allow(static_mut_refs)]
#![feature(core_intrinsics)]
#![feature(format_args_nl)]
#![feature(int_roundings)]
#![feature(sync_unsafe_cell)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

mod bsp;
mod common;
mod console;
mod cpu;
mod driver;
mod exception;
mod memory;
mod panic_wait;
mod print;
mod synchronization;
mod time;

///
/// Initialize the kernel
/// The init calls in this function must appear in the correct order
/// MMU + Data caching must be activated before anything else
/// Without it, any atomic operations like the soon-to-arrive spinlocks in the device drivers will notwork properly on RPi SoCs
fn kernel_init() -> ! {
  use memory::mmu::interface::MMU;

  exception::handling_init();

  if let Err(string) = unsafe { memory::mmu::mmu().enable_mmu_and_caching() } {
    panic!("MMU: {}", string);
  }

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
  use core::time::Duration;

  info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
  info!("Booting on: {}", bsp::board_name());

  info!("MMU online; special regions:");
  bsp::memory::mmu::virt_mem_layout().print_layout();

  let (_, privilege_level) = exception::current_privilege_level();
  info!("Current privilege level: {}", privilege_level);

  info!("Exception handling state:");
  exception::asynchronous::print_state();

  info!("Architectural timer resolution: {} ns", time::time_manager().resolution().as_nanos());
  info!("Drivers loaded:");
  driver::driver_manager().enumerate();

  info!("Timer test: spinning for 1 second");
  time::time_manager().spin_for(Duration::from_secs(1));

  // Cause an exception by accessing a virtual address for which no translation was setup
  // This code accesses the address 8 GiB, which is outside the mapped address space
  //
  // For demo purposes, the exception handler will catch the faulting 8 GiB address and allow execution to continue
  info!("");
  info!("Trying to read from address 8 GiB");
  let big_addr: u64 = 8 * 1024 * 1024 * 1024;
  unsafe { core::ptr::read_volatile(big_addr as *mut u64) };

  info!("************************************************");
  info!("WHOA! We recovered from a synchronous exception!");
  info!("************************************************");
  info!("");
  info!("... let's try again");

  // Now use address 9 GiB
  // The exception handler won't forgive us this time...
  info!("Trying to read from address 9 GiB");
  let big_addr: u64 = 9 * 1024 * 1024 * 1024;
  unsafe { core::ptr::read_volatile(big_addr as *mut u64) };
  
  // We won't get here right now
  info!("Echoing input now");

  // Discard any spurious received characters before going into echo mode
  console().clear_rx();
  loop {
    let c = console().read_char();
    console().write_char(c);
  }
}