// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

use core::cell::SyncUnsafeCell;

pub mod interface {
  pub trait Mutex {
    type Data;

    fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R;
  }
}

pub struct NullLock<T>
where T: ?Sized {
  data: SyncUnsafeCell<T>,
}

impl<T> NullLock<T> {
  pub const fn new(data: T) -> Self {
    Self {
      data: SyncUnsafeCell::new(data),
    }
  }
}

impl<T> interface::Mutex for NullLock<T> {
  type Data = T;

  fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R {
    let data = unsafe { &mut *self.data.get() };

    f(data)
  }
}