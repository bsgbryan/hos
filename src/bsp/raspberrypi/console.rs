// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! BSP console utilities

use core::fmt;

use crate::{
  console,
  synchronization::{
    NullLock,
    self,
  },
};

struct QEMUOutputInner {
  chars_written: usize,
}

pub struct QEMUOutput {
  inner: NullLock<QEMUOutputInner>,
}

unsafe impl Sync for QEMUOutput {}

static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();

impl QEMUOutputInner {
  const fn new() -> QEMUOutputInner {
    QEMUOutputInner { chars_written: 0 }
  }

  fn write_char(&mut self, c: char) {
    unsafe {
      core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
    }

    self.chars_written += 1;
  }
}

impl fmt::Write for QEMUOutputInner {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    for c in s.chars() {
      if c == '\n' {
        self.write_char('\r');
      }

      self.write_char(c);
    }

    Ok(())
  }
}

impl QEMUOutput {
  pub const fn new() -> QEMUOutput {
    QEMUOutput { inner: NullLock::new(QEMUOutputInner::new()) }
  }
}

pub fn console() -> &'static dyn console::interface::All {
  &QEMU_OUTPUT
}

use synchronization::interface::Mutex;

impl console::interface::Write for QEMUOutput {
  fn write_fmt(&self, args: fmt::Arguments<'_>) -> fmt::Result {
      self.inner.lock(|i| fmt::Write::write_fmt(i, args))
  }
}

impl console::interface::Statistics for QEMUOutput {
  fn char_written(&self) -> usize {
    self.inner.lock(|i| i.chars_written)
  }
}

impl console::interface::All for QEMUOutput {}
