// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Synchronous and asynchronous exception handling

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/exception.rs"]
mod arch_exception;

pub mod asynchronous;

pub use arch_exception::{
  current_privilege_level,
  handling_init,
};

/// Processing Element privilege levels
#[derive(Eq, PartialEq)]
pub enum PrivilegeLevel {
  User,
  Kernel,
  Hypervisor,
  Unknown,
}