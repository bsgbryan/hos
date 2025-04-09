// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! GPIO driver

use core::time::Duration;

use tock_registers::{
  interfaces::{
    ReadWriteable,
    Writeable,
  },
  register_bitfields,
  register_structs,
  registers::ReadWrite,
};

use crate::{
  bsp::device_driver::common::MMIODerefWrapper,
  driver,
  synchronization::{
    NullLock,
    self,
  },
  time,
};

register_bitfields! {
  u32,

  /// GPIO Function Select 1
  GPFSEL1 [
    /// Pin 15
    FSEL15 OFFSET(15) NUMBITS(3) [
      Input    = 0b000,
      Output   = 0b001,
      AltFunc0 = 0b100 // PL011 UART RX
    ],

    /// Pin 14
    FSEL14 OFFSET(12) NUMBITS(3) [
      Input    = 0b000,
      Output   = 0b001,
      AltFunc0 = 0b100 // PL011 UART TX
    ]
  ],

  /// GPIO Pull-up/down Register
  /// BCM2837 only
  GPPUD [
    /// Controls the actuation of the internal pull-up-down control line to ALL the GPIO pins
    PUD OFFSET(0) NUMBITS(2) [
      Off      = 0b00,
      PullDown = 0b01,
      PullUp   = 0b10
    ]
  ],

  /// GPIO Pull-up/down Clock Register 0
  /// BCM2837 only
  GPPUDCLK0 [
    /// Pin 15
    PUDCLK15 OFFSET(15) NUMBITS(1) [
      NoEffect    = 0,
      AssertClock = 1
    ],

    /// Pin 14
    PUDCLK14 OFFSET(14) NUMBITS(1) [
      NoEffect    = 0,
      AssertClock = 1
    ]
  ],

  /// GPIO Pull-up / Pull-down Register 0
  /// BCM2711 only
  GPIO_PUP_PDN_CNTRL_REG0 [
    /// Pin 15
    GPIO_PUP_PDN_CNTRL15 OFFSET(30) NUMBITS(2) [
      NoResistor = 0b00,
      PullUp     = 0b01
    ],

    /// Pin 14
    GPIO_PUP_PDN_CNTRL14 OFFSET(28) NUMBITS(2) [
      NoResistor = 0b00,
      PullUp     = 0b01
    ]
  ]
}

register_structs! {
  #[allow(non_snake_case)]
  RegisterBlock {
    (0x00 => _reserved1),
    (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
    (0x08 => _reserved2),
    (0x94 => GPPUD: ReadWrite<u32, GPPUD::Register>),
    (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
    (0x9c => _reserved3),
    (0xe4 => GPIO_PUP_PDN_CNTRL_REG0: ReadWrite<u32, GPIO_PUP_PDN_CNTRL_REG0::Register>),
    (0xe8 => @END),
  }
}

/// Abstraction for the associated MMIO registers
type Registers = MMIODerefWrapper<RegisterBlock>;

struct GPIOInner {
  registers: Registers,
}

/// Representation of the GPIO hardware
pub struct GPIO {
  inner: NullLock<GPIOInner>,
}

unsafe impl Sync for GPIO {}

impl GPIOInner {
  /// Create an instance
  /// 
  /// # Safety
  /// - The caller must ensure a valid MMIO start address is specified
  pub const unsafe fn new(mmio_start_addr: usize) -> Self {
    Self {
      registers: unsafe { Registers::new(mmio_start_addr) },
    }
  }

  /// Disable pull-up/down on lins 14 and 15
  #[cfg(feature = "bsp_rpi3")]
  fn disable_pud_14_15_bcm2837(&mut self) {
    // The Linu x2837 GPIO driver waits 1 µs between steps
    const DELAY: Duration = Duration::from_micros(1);

    self.registers.GPPUD.write(GPPUD::PUD::Off);
    time::time_manager().spin_for(DELAY);

    self.registers.GPPUDCLK0.write(
      GPPUDCLK0::PUDCLK15::AssertClock
      +
      GPPUDCLK0::PUDCLK14::AssertClock
    );
    time::time_manager().spin_for(DELAY);

    self.registers.GPPUD.write(GPPUD::PUD::Off);
    self.registers.GPPUDCLK0.set(0);
  }

  /// Disable pull-up/down on pins 14 and 15
  #[cfg(feature = "bsp_rpi4")]
  fn disable_pud_14_15_bcm2711(&mut self) {
    self.registers.GPIO_PUP_PDN_CNTRL_REG0.write(
      GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::PullUp
      +
      GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::PullUp
    );
  }

  /// Map PL011 UART as standard output
  /// 
  /// USB TX -> pin 14 (Pi RX)
  /// USB RX -> pin 15 (Pi TX)
  pub fn map_pl011_uart(&mut self) {
    // Select the function for the pins
    self.registers.GPFSEL1.modify(
      GPFSEL1::FSEL15::AltFunc0
      +
      GPFSEL1::FSEL14::AltFunc0
    );

    // Disable pul-up/down on pins 14 and 15
    #[cfg(feature = "bsp_rpi3")]
    self.disable_pud_14_15_bcm2837();
    #[cfg(feature = "bsp_rpi4")]
    self.disable_pud_14_15_bcm2711();
  }
}

impl GPIO {
  pub const COMPATIBLE: &'static str = "BCM GPIO";

  /// Create and instance
  /// 
  /// # Safety
  /// - The caller must provide a valid MMIO start address
  pub const unsafe fn new(mmio_start_addr: usize) -> Self {
    Self {
      inner: NullLock::new(unsafe { GPIOInner::new(mmio_start_addr) }),
    }
  }

  /// Concurrency-safe version of `GPIOInner.map_pl011_uart()`
  pub fn map_pl011_uart(&self) {
    self.inner.lock(|i| i.map_pl011_uart())
  }
}

use synchronization::interface::Mutex;

impl driver::interface::DeviceDriver for GPIO {
  fn compatible(&self) -> &'static str { Self::COMPATIBLE }
}
