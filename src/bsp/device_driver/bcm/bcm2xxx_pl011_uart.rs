// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! PL011 UART driver

use core::fmt;

use tock_registers::{
  interfaces::{
    Readable,
    Writeable,
  },
  register_bitfields,
  register_structs,
  registers::{
    ReadOnly,
    ReadWrite,
    WriteOnly,
  },
};

use crate::{
  bsp::device_driver::common::MMIODerefWrapper,
  console,
  cpu,
  driver,
  synchronization::{
    NullLock,
    self,
  },
};

register_bitfields! {
  u32,

  /// Flag Register
  FR [
    /// Transmit FIFO empty
    /// The meaning of this bit depends of the state of the LCR_H::FEN bit
    /// - If FIFO is disabled, this bit (the TXFE bit) is set when the transmit holding register is empty
    /// - If FIFO is enabled, this bit (the TXFE bit) is set when the transmit FIFO is empty
    /// - This bit not indicate if there is data in the transmit shift register
    TXFE OFFSET(7) NUMBITS(1) [],

    /// Transmit FIFO full
    /// The meaning if this bit depends on the state of the LCR_H::FEN bit
    /// - If FIFO is disabled, this bit (the TXFF bit) is set when the transmit holding register is full
    /// - If FIFO is enabled, this bit (the TXFF bit) is set the transmit FIFO is full
    TXFF OFFSET(5) NUMBITS(1) [],

    /// Recieve FIFO empty
    /// The meaning of this bit depends of the state of the LCR_H::FEN bit
    /// - If FIFO is disabled, this bit (the RXFE bit) is set when the receive holding register is empty
    /// - If FIFO is enabled, this bit (the RXFE bit) is set when the recieve FIFO is empty
    RXFE OFFSET(4) NUMBITS(1) [],

    /// UART busy
    /// IF this bit is set to 1 the UART is busy transmitting data
    /// This bit remains set until the complete byte, including all the stop bits, has been sent from the shift register
    /// This bit is set as soon as the transmit FIFO becomes non-empty - regardless of whether the UART is enabled or not
    BUSY OFFSET(3) NUMBITS(1) [],
  ],

  /// Integer Baud Rate Divisor
  IBRD [
    /// The value
    BAUD_DIVINT OFFSET(0) NUMBITS(16) []
  ],

  /// Fractional Baud Rate Divisor
  FBRD [
    /// The value
    BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
  ],

  /// Line Control Register
  LCR_H [
    /// Word length
    /// These bits indicate the number of data bits transmitted or received in a frame
    #[allow(clippy::enum_variant_names)]
    WLEN OFFSET(5) NUMBITS(2) [
      FiveBit  = 0b00,
      SixBit   = 0b01,
      SevenBit = 0b10,
      EightBit = 0b11
    ],

    /// Enable FIFO
    /// 0 = FIFO is disabled
    /// 1 = FIFO is enabled
    FEN OFFSET(4) NUMBITS(1) [
      FifoDisabled = 0,
      FifoEnabled  = 1
    ]
  ],

  /// Control Register
  CR [
    /// Receive enabled
    /// When this is set to 1 the receive section of the UART is enabled
    /// Data reception occurs for either UART signals or SIR signals, depending of the setting of the SIREN bit
    /// When the UART is disabled in the middle of reception it completes the current character before stopping
    RXE OFFSET(9) NUMBITS(1) [
      Disabled = 0,
      Enabled = 1
    ],

    /// Transmit enabled
    /// If this bit is set to 1 the transmit section of the UART is enabled
    /// Data transmission occurs for either UART signals or SIR signals, depending on the setting of the SIREN bit
    /// When the UART is disabled in the middle of transmission it completes the current character before stopping
    TXE OFFSET(8) NUMBITS(1) [
      Disabled = 0,
      Enabled = 1
    ],

    /// UART enabled
    /// 0 = UART is disabled: If the UART is disabled in the middle of transmission or reception it completes the cirrent character before stopping
    /// 1 = UART is enabled: Data transmission and reception occurs for either UART or SIR signals, depending on the setting of the SIREN bit
    UARTEN OFFSET(0) NUMBITS(1) [
      Disabled = 0,
      Enabled = 1
    ]
  ],

  /// Interrupt Clear Register
  ICR [
    /// Meta field for all pending interrupts
    ALL OFFSET(0) NUMBITS(11) []
  ]
}

register_structs! {
  #[allow(non_snake_case)]
  pub RegisterBlock {
    (0x00 => DR: ReadWrite<u32>),
    (0x04 => _reserved1),
    (0x18 => FR: ReadOnly<u32, FR::Register>),
    (0x1c => _reserved2),
    (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
    (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
    (0x2c => LCR_H: WriteOnly<u32, LCR_H::Register>),
    (0x30 => CR: WriteOnly<u32, CR::Register>),
    (0x34 => _reserved3),
    (0x44 => ICR: WriteOnly<u32, ICR::Register>),
    (0x48 => @END),
  }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

#[derive(PartialEq)]
enum BlockingMode {
  Blocking,
  NonBlocking,
}

struct PL011UartInner {
  registers: Registers,
  chars_written: usize,
  chars_read: usize,
}

pub struct PL011Uart {
  inner: NullLock<PL011UartInner>,
}

unsafe impl Sync for PL011Uart {}

impl PL011UartInner {
  /// Create an instance
  /// 
  /// # Safety
  /// 
  /// - The caller must provide a valid MMIO start address
  pub const fn new(mmio_start_addr: usize) -> Self {
    Self {
      registers: unsafe { Registers::new(mmio_start_addr) },
      chars_written: 0,
      chars_read: 0,
    }
  }

  /// Setup buad rate and characteristics
  /// 
  /// This results in 8N1 and 921_600 baud
  /// The calculation for the BRD is (we set the clocks to 48 MHz in config.txt): `(48_000_000 / 16) / 921_600 = 3.2552083`
  /// This means `3` (the integer part) goes into `IBRD`
  /// The `FBRD` calculation (according to the PL011 Technical Reference Manual) is: `INTEGER((0.2552083 * 64) + 0.5) = 16`
  /// So the generated baud rate divisor is `3 + 16/64 = 3.25` - which results in a generated baud rate of `48_000_000 / (16 * 3.25) = 923_077`
  pub fn init(&mut self) {
    // Execution can arrive here while there are still characters queued in the TX FIFO and actively being sent out by the UART hardware
    // If the UART is turned of in such a case, the characters will be lost
    // This can happen at runtime on a call to `panic!` because `panic!` initializes its own UART instance
    // So, we flush first to ensure any/all pending characters are transmitted
    self.flush();

    // Turn the UART off temporarily
    self.registers.CR.set(0);

    // Clear all pending interrupts
    self.registers.ICR.write(ICR::ALL::CLEAR);

    // From the PL011 Technical Reference Manual:
    // The LCR_H, IBRD, and FBRD registers form the single 30-bit wide LCR Register that is updated on a single write strobe generated by a LCR_H write
    // So to internally update the contents fothe IBRD or FBRD an LCR_H write must always be performed at the end
    self.registers.IBRD.write(IBRD::BAUD_DIVINT.val(3));
    self.registers.FBRD.write(FBRD::BAUD_DIVFRAC.val(16));
    self.registers.LCR_H.write(
      LCR_H::WLEN::EightBit
      +
      LCR_H::FEN::FifoEnabled
    );

    // Turn the UART on
    self.registers.CR.write(
      CR::UARTEN::Enabled
      +
      CR::TXE::Enabled
      +
      CR::RXE::Enabled
    );
  }

  /// Send a character
  fn write_char(&mut self, c: char) {
    // Spin while TX FIFO full is set; waiting for an empty slot
    while self.registers.FR.matches_all(FR::TXFF::SET) {
      cpu::nop();
    }

    // Write the character to the buffer
    self.registers.DR.set(c as u32);

    self.chars_written += 1;
  }

  /// Block execution until the last buffered character has been physically put in the TX wire
  fn flush(&self) {
    // Spin until the busy bit is cleared
    while self.registers.FR.matches_all(FR::BUSY::SET) {
      cpu::nop();
    }
  }

  /// Retrieve a character
  fn read_char(&mut self, blocking_mode: BlockingMode) -> Option<char> {
    // If RX FIFO is empty
    if self.registers.FR.matches_all(FR::RXFE::SET) {
      // immediately return in non-blocking mode
      if blocking_mode == BlockingMode::NonBlocking {
        return None;
      }

      // Otherwise, wait until a char is received
      while self.registers.FR.matches_all(FR::RXFE::SET) {
        cpu::nop();
      }
    }

    // Read one character
    let ret = self.registers.DR.get() as u8 as char;

    self.chars_read += 1;

    Some(ret)
  }
}

/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros - whic hare used to implement the `kernel`'s `print!` and `println!` macros
/// By immplementing `write_str` we get `write_fmt` automatically
/// This function takes a `&mut self` so it must be implemented on the inner struct
impl fmt::Write for PL011UartInner {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    for c in s.chars() { self.write_char(c); }

    Ok(())
  }
}

impl PL011Uart {
  pub const COMPATIBLE: &'static str = "BCM PL011 UART";

  /// Create an instance
  /// 
  /// # Safety
  /// 
  /// - The caller must provide a valid MMIO start address
  pub const unsafe fn new(mmio_start_addr: usize) -> Self {
    Self {
      inner: NullLock::new(PL011UartInner::new(mmio_start_addr)),
    }
  }
}

use synchronization::interface::Mutex;

impl driver::interface::DeviceDriver for PL011Uart {
  fn compatible(&self) -> &'static str {
    Self::COMPATIBLE
  }

  unsafe fn init(&self) -> Result<(), &'static str> {
    self.inner.lock(|i| i.init());

    Ok(())
  }
}

impl console::interface::Write for PL011Uart {
  fn write_char(&self, c: char) {
    self.inner.lock(|i| i.write_char(c));
  }

  fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
    self.inner.lock(|i| fmt::Write::write_fmt(i, args))
  }

  fn flush(&self) {
    self.inner.lock(|i| i.flush());
  }
}

impl console::interface::Read for PL011Uart {
  fn read_char(&self) -> char {
    self.inner.lock(|i| i.read_char(BlockingMode::Blocking).unwrap())
  }

  fn clear_rx(&self) {
    // Read from ther RX FIFO until it's empty
    while self.inner.lock(|i| i.read_char(BlockingMode::NonBlocking)).is_some() {}
  }
}

impl console::interface::Statistics for PL011Uart {
  // fn chars_written(&self) -> usize {
  //   self.inner.lock(|i| i.chars_written)
  // }

  // fn chars_read(&self) -> usize {
  //   self.inner.lock(|i| i.chars_read)
  // }
}

impl console::interface::All for PL011Uart {}