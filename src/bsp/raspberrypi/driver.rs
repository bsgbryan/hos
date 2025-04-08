// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! RPi drivers

use core::sync::atomic::{
  AtomicBool,
  Ordering,
};

use crate::{
  bsp::device_driver,
  console,
  driver as generic_driver,
};

use super::memory::map::mmio;

static PL011_UART: device_driver::PL011Uart = unsafe { device_driver::PL011Uart::new(mmio::PL011_UART_START) };

static GPIO: device_driver::GPIO = unsafe { device_driver::GPIO::new(mmio::GPIO_START) };

// This must only be called after a succesful UART driver init
fn post_uart_init() -> Result<(), &'static str> {
  console::register_console(&PL011_UART);
  Ok(())
}

// This must only be called after a successful GPIO driver init
fn post_gpio_init() -> Result<(), &'static str> {
  GPIO.map_pl011_uart();
  Ok(())
}

fn driver_uart() -> Result<(), &'static str> {
  let uart_descriptor = generic_driver::DeviceDriverDescriptor::new(
    &PL011_UART,
    Some(post_uart_init),
  );

  generic_driver::driver_manager().register_driver(uart_descriptor);

  Ok(())
}

fn driver_gpio() -> Result<(), &'static str> {
  let gpio_descriptor = generic_driver::DeviceDriverDescriptor::new(
    &GPIO,
    Some(post_gpio_init),
  );

  generic_driver::driver_manager().register_driver(gpio_descriptor);

  Ok(())
}

pub unsafe fn init() -> Result<(), &'static str> {
  static INIT_DONE: AtomicBool = AtomicBool::new(false);

  if INIT_DONE.load(Ordering::Relaxed) {
    return Err("Already initialized");
  }

  driver_uart()?;
  driver_gpio()?;

  INIT_DONE.store(true, Ordering::Relaxed);

  Ok(())
}
