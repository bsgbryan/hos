// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Timer primitives

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/time.rs"]
mod arch_time;

use core::time::Duration;

pub struct TimeManager;

static TIME_MANAGER: TimeManager = TimeManager::new();

pub fn time_manager() -> &'static TimeManager {
  &TIME_MANAGER
}

impl TimeManager {
  /// Create an instance
  pub const fn new() -> Self { Self }

  /// The timer's resolution
  pub fn resolution(&self) -> Duration {
    arch_time::resolution()
  }

  /// The duration since device power-on
  /// This includes time consumed by firmware and bootloaders
  pub fn uptime(&self) -> Duration {
    arch_time::uptime()
  }

  /// Spin for a given duration
  pub fn spin_for(&self, duration: Duration) {
    arch_time::spin_for(duration);
  }
}