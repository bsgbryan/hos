// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Common device driver code

use core::{
  marker::PhantomData,
  ops,
};

pub struct MMIODerefWrapper<T> {
  start_addr: usize,
  phantom: PhantomData<fn() -> T>,
}

impl<T> MMIODerefWrapper<T> {
  /// Create an instance
  /// 
  /// # Safety
  /// 
  /// Caller must provide a valid MMIO start address
  pub const unsafe fn new(start_addr: usize) -> Self {
    Self {
      start_addr,
      phantom: PhantomData,
    }
  }
}

impl<T> ops::Deref for MMIODerefWrapper<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*(self.start_addr as *const _) }
  }
}