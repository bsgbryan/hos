// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Architectural synchronous and asynchronous exception handling
//! 
//! # Overview
//! 
//! Since modules are imported into generic modules using the path attribute,
//! the path of this file is: `crate::exception::arch_asynchronous`

use core::{
  arch::global_asm,
  cell::UnsafeCell,
  fmt::{
    Display,
    Formatter,
    Result,
  },
};

use aarch64_cpu::{
  asm::barrier,
  registers::*,
};  
use tock_registers::{
  interfaces::{
    Readable,
    Writeable,
  },
  registers::InMemoryRegister,
};

use crate::exception::PrivilegeLevel;

global_asm!(include_str!("exception.s"));

/// Wrapper struct for memory copies of registers
#[repr(transparent)]
struct SpsrEL1(InMemoryRegister<u64, SPSR_EL1::Register>);

/// Human-readable SPSR_EL1
impl Display for SpsrEL1 {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    // Raw value
    writeln!(f, "SPSR_EL1: {:#010x}", self.0.get())?;

    let to_flag_str = |x| -> &'static str {
      if x { "Set" } else { "Not set" }
    };

    writeln!(f, "Flags:")?;
    writeln!(f, "Negative (N): {}", to_flag_str(self.0.is_set(SPSR_EL1::N)))?;
    writeln!(f, "Zero     (Z): {}", to_flag_str(self.0.is_set(SPSR_EL1::Z)))?;
    writeln!(f, "Carry    (C): {}", to_flag_str(self.0.is_set(SPSR_EL1::C)))?;
    writeln!(f, "Overflow (V): {}", to_flag_str(self.0.is_set(SPSR_EL1::V)))?;

    let to_mask_str = |x| -> &'static str {
      if x { "Masked" } else { "Unmasked" }
    };

    writeln!(f, "Exception Handling State:")?;
    writeln!(f, "Debug  (D): {}", to_mask_str(self.0.is_set(SPSR_EL1::D)))?;
    writeln!(f, "SError (A): {}", to_mask_str(self.0.is_set(SPSR_EL1::A)))?;
    writeln!(f, "IRQ    (I): {}", to_mask_str(self.0.is_set(SPSR_EL1::I)))?;
    writeln!(f, "FIQ    (F): {}", to_mask_str(self.0.is_set(SPSR_EL1::F)))?;

    write!(f, "Illegal Execution State (IL): {}", to_flag_str(self.0.is_set(SPSR_EL1::IL)))
  }
}

/// Wrapper struct for memory copies of registers
struct EsrEL1(InMemoryRegister<u64, ESR_EL1::Register>);

impl EsrEL1 {
  #[inline(always)]
  fn exception_class(&self) -> Option<ESR_EL1::EC::Value> {
    self.0.read_as_enum(ESR_EL1::EC)
  }
}

/// Human-readable ESR_EL1
impl Display for EsrEL1 {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    // Raw print of entire register
    writeln!(f, "ESR_EL1: {:#010x}", self.0.get())?;

    // Raw print of Exception Class
    write!(f, "Exception Class (EC): {:#x}", self.0.read(ESR_EL1::EC))?;

    // Exception Class
    let ec_translation = match self.exception_class() {
      Some(ESR_EL1::EC::Value::DataAbortCurrentEL) => "Data Abort; current EL",
      _ => "N/A",
    };
    writeln!(f, " - {}", ec_translation)?;

    // Raw print of instruction specific syndrome
    write!(f, " Instruction Specific Syndrome (ISS): {:#x}", self.0.read(ESR_EL1::ISS))
  }
}

/// The exception context as it is stored on the stack on exception entry
#[repr(C)]
struct ExceptionContext {
  /// General Purpose Registers
  gpr: [u64; 30],

  /// The Link Register; aka x30
  lr: u64,

  /// Exception Link Register; The Program Counter atthe time the exception happened
  elr_el1: u64,

  /// Saved Program Status
  spsr_el1: SpsrEL1,

  /// Exception Synrome Register
  esr_el1: EsrEL1,
}

impl ExceptionContext {
  #[inline(always)]
  fn exception_class(&self) -> Option<ESR_EL1::EC::Value> {
    self.esr_el1.exception_class()
  }

  #[inline(always)]
  fn fault_address_valid(&self) -> bool {
    use ESR_EL1::EC::Value::*;

    match self.exception_class() {
      None => false,
      Some(ec) => matches!(
        ec,
        InstrAbortLowerEL   |
        InstrAbortCurrentEL |
        PCAlignmentFault    |
        DataAbortLowerEL    |
        DataAbortCurrentEL  |
        WatchpointLowerEL   |
        WatchpointCurrentEL
      )
    }
  }
}

/// Human readable printout of the ExceptionContext
impl Display for ExceptionContext {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    writeln!(f, "{}", self.esr_el1)?;

    if self.fault_address_valid() {
      writeln!(f, "FAR_EL1 {:#018x}", FAR_EL1.get() as usize)?;
    }

    writeln!(f, "{}", self.spsr_el1)?;
    writeln!(f)?;
    writeln!(f, "General Purpose Registers:")?;

    let alternating = |x| -> &'static str {
      if x % 2 == 0 { "   " } else { "\n" }
    };

    // Print two registers per line
    for (i, reg) in self.gpr.iter().enumerate() {
      write!(f, "x{: <2}: {: >#018x}{}", i, reg, alternating(i))?;
    }

    write!(f, "lr: {:#018x}", self.lr)
  }
}

/// Prints verbose infomration about the exception and then panics
fn default_exception_handler(exc: &ExceptionContext) {
  panic!("CPU Exception:\n\n{}", exc);
}

#[unsafe(no_mangle)]
extern "C" fn current_el0_synchronous(_e: &mut ExceptionContext) {
  panic!("Should not be here: Use of SP_EL0 is EL1 is not supported")
}

#[unsafe(no_mangle)]
extern "C" fn current_el0_irq(_e: &mut ExceptionContext) {
  panic!("Should not be here: Use of SP_EL0 is EL1 is not supported")
}
#[unsafe(no_mangle)]
extern "C" fn current_el0_serror(_e: &mut ExceptionContext) {
  panic!("Should not be here: Use of SP_EL0 is EL1 is not supported")
}

#[unsafe(no_mangle)]
extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
  if e.fault_address_valid() {
    let far_el1 = FAR_EL1.get();

    // This catches the demo case for this tutorial
    // If the fault address happens to be 8GiB,
    // advance the exception link register by one instruction so that execution can continue
    if far_el1 == 8 * 1024 * 1024 * 1024 {
      e.elr_el1 += 4;

      return;
    }
  }

  default_exception_handler(e);
}

#[unsafe(no_mangle)]
extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[unsafe(no_mangle)]
extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[unsafe(no_mangle)]
extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[unsafe(no_mangle)]
extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[unsafe(no_mangle)]
extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[unsafe(no_mangle)]
extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[unsafe(no_mangle)]
extern "C" fn lower_aarch32_irq(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[unsafe(no_mangle)]
extern "C" fn lower_aarch32_serror(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

/// The Processing Element's current privilege level
pub fn current_privilege_level() -> (PrivilegeLevel, &'static str) {
  let el = CurrentEL.read_as_enum(CurrentEL::EL);

  match el {
    Some(CurrentEL::EL::Value::EL2) => (PrivilegeLevel::Hypervisor, "EL2"    ),
    Some(CurrentEL::EL::Value::EL1) => (PrivilegeLevel::Kernel,     "EL1"    ),
    Some(CurrentEL::EL::Value::EL0) => (PrivilegeLevel::User,       "EL0"    ),
    _                               => (PrivilegeLevel::Unknown,    "Unknown"),
  }
}

/// Initialize exception handling by setting the Exception Vector Base Address Register
/// 
/// # Safety
/// 
/// - Changes the hardware state of the executing core
/// - The vector table and the symbol `__exception_vector_table_start` from the assembly script must adhere to the alignment and size constraints demanded by the ARMv8-A Architecture Rerence Manual
pub fn handling_init() {
  // Provided by exception.s
  unsafe extern "Rust" {
    unsafe static __exception_vector_table_start: UnsafeCell<()>;
  }

  VBAR_EL1.set(unsafe { __exception_vector_table_start.get() as u64 });

  // Force VBAR update to complete before next instruction
  barrier::isb(barrier::SY);
}
