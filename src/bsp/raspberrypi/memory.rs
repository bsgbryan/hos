// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! RPi memory management
//!
//! The physical memory layout.
//!
//! The Raspberry's firmware copies the kernel binary to 0x8_0000. The preceding region will be used
//! as the boot core's stack.
//!
//! +---------------------------------------+
//! |                                       | 0x0
//! |                                       |                                ^
//! | Boot-core Stack                       |                                | stack
//! |                                       |                                | growth
//! |                                       |                                | direction
//! +---------------------------------------+
//! |                                       | code_start @ 0x8_0000
//! | .text                                 |
//! | .rodata                               |
//! | .got                                  |
//! |                                       |
//! +---------------------------------------+
//! |                                       | code_end_exclusive
//! | .data                                 |
//! | .bss                                  |
//! |                                       |
//! +---------------------------------------+
//! |                                       |
//! |                                       |

pub mod mmu;

use core::cell::UnsafeCell;

// Symbols for the linker script
unsafe extern "Rust" {
  static __code_start:         UnsafeCell<()>;
  static __code_end_exclusive: UnsafeCell<()>;
}

// The board's physical memory map
pub(super) mod map {
  /// The inclusive end address of the memory map
  /// End address + 1 must be a power of 2
  /// 
  /// # Note
  /// 
  /// RPi3 and RPi4 boards can have different amounts of RAM
  /// To make our code lean, for educational purposes, we set the map size of the address space to 4GiB regadless of board
  /// This way we can map the entire range that we need (end of MMIO for RPi4) in one take
  /// However, making this tradeoff has the downside of making it possible for the CPU toassert a physical address that is not backed by any DRAM
  /// (ie: accessing an address close to 4GiB on an RPi3 that comes with only 1GiB of RAM)
  /// This would result in a crash or other kind of error
  pub const END_INCLUSIVE: usize = 0xFFFF_FFFF;
  pub const GPIO_OFFSET:   usize = 0x0020_0000;
  pub const UART_OFFSET:   usize = 0x0020_1000;

  /// Physical devices
  #[cfg(feature = "bsp_rpi3")]
  pub mod mmio {
    use super::*;

    pub const START:            usize = 0x3F00_0000;
    pub const GPIO_START:       usize = START + GPIO_OFFSET;
    pub const PL011_UART_START: usize = START + UART_OFFSET;
    pub const END_INCLUSIVE:    usize = 0x4000_FFFF;
  }

  /// Physical devices
  #[cfg(feature = "bsp_rpi4")]
  pub mod mmio {
    use super::*;

    pub const START:            usize = 0xFE00_0000;
    pub const GPIO_START:       usize = START + GPIO_OFFSET;
    pub const PL011_UART_START: usize = START + UART_OFFSET;
    pub const END_INCLUSIVE:    usize = 0xFF84_FFFF;
  }
}

/// Start page address of the code segment
/// 
/// # Safety
/// 
/// Value is provided by the linker script and must be trusted as-is
#[inline(always)]
fn code_start() -> usize { unsafe { __code_start.get() as usize } }

/// Exclusive end page address of the code segment
/// 
/// # Safety
/// 
/// Value is provided by the linker script and must be trusted as-is
#[inline(always)]
fn code_end_exclusive() -> usize { unsafe { __code_end_exclusive.get() as usize } }
