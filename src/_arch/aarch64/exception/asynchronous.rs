// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Architectural asynchronous exception handling
//! 
//! # Overview
//! 
//! Since modules are imported into generic modules using the path attribute,
//! the path of this file is: `crate::exception::asynchronous::arch_asynchronous`

use aarch64_cpu::registers::*;
use tock_registers::interfaces::Readable;

trait DaifField {
  fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register>;
}

struct Debug;
struct SError;
struct IRQ;
struct FIQ;

impl DaifField for Debug {
  fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> { DAIF::D }
}

impl DaifField for SError {
  fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> { DAIF::A }
}

impl DaifField for IRQ {
  fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> { DAIF::I }
}

impl DaifField for FIQ {
  fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> { DAIF::F }
}

fn is_masked<T>() -> bool
where T: DaifField {
  DAIF.is_set(T::daif_field())
}

/// Print the AArch64 exception status
pub fn print_state() {
  use crate::info;

  let to_mask_str = |s| -> _ { if s { "Masked"} else { "Unmasked"} };

  info!("\tDebug:  {}", to_mask_str(is_masked::<Debug>()));
  info!("\tSError: {}", to_mask_str(is_masked::<SError>()));
  info!("\tIRQ:    {}", to_mask_str(is_masked::<IRQ>()));
  info!("\tFIQ:    {}", to_mask_str(is_masked::<FIQ>()));
}