// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Architectural synchronous and asynchronous exception handling
//! 
//! # Overview
//! 
//! Since modules are imported into generic modules using the path attribute,
//! the path of this file is: `crate::exception::arch_asynchronous`

use aarch64_cpu::registers::*;
use tock_registers::interfaces::Readable;

use crate::exception::PrivilegeLevel;

/// The Processing Element's current privilege level
pub fn current_privilege_level() -> (PrivilegeLevel, &'static str) {
  let el = CurrentEL.read_as_enum(CurrentEL::EL);

  match el {
    Some(CurrentEL::EL::Value::EL2) => (PrivilegeLevel::Hypervisor, "EL2"),
    Some(CurrentEL::EL::Value::EL1) => (PrivilegeLevel::Kernel, "EL1"),
    Some(CurrentEL::EL::Value::EL0) => (PrivilegeLevel::User, "EL0"),
    _ => (PrivilegeLevel::Unknown, "Unknown"),
  }
}