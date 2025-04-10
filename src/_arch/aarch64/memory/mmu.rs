// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Memory Management Unit driver
//! 
//! Only 64KiB granule is supported
//! 
//! # Overview
//! 
//! Since arch modules are imported into generic modules using the path attribute,
//! the path of this file is: `crate::memory::mmu::arch_mmu`

use core::intrinsics::unlikely;

use aarch64_cpu::{
  asm::barrier,
  registers::*,
};
use tock_registers::interfaces::{
  ReadWriteable,
  Readable,
  Writeable,
};

use crate::{
  bsp,
  memory,
  memory::mmu::{
    translation_table::KernelTranslationTable,
    TranslationGranule,
  },
};

struct MemoryManagementUnit;

pub type Granule512MiB = TranslationGranule<{ 512 * 1024 * 1024 }>;
pub type Granule64KiB  = TranslationGranule<{  64 * 1024        }>;

/// Constants for indexing into the MAIR_EL1
#[allow(dead_code)]
pub mod mair {
  pub const DEVICE: u64 = 0;
  pub const NORMAL: u64 = 1;
}

/// The kernel translation tables
/// 
/// # Safety
/// 
/// Supposed to land in `.bss` so ensure that all initial member values boil down to `0`
static mut KERNEL_TABLES: KernelTranslationTable = KernelTranslationTable::new();

static MMU: MemoryManagementUnit = MemoryManagementUnit;

impl<const AS_SIZE: usize> memory::mmu::AddressSpace<AS_SIZE> {
  /// Checks for architectural restrictions
  pub const fn arch_address_space_size_sanity_checks() {
    assert!((AS_SIZE % Granule512MiB::SIZE) == 0);

    // Check for 48 bit virtual address size as maximum;
    // which is supported by any ARMv8 version
    assert!(AS_SIZE <= (1 << 48));
  }
}

impl MemoryManagementUnit {
  /// Serup function for the MAIR_EL1 register
  fn set_up_mair(&self) {
    // Define the memory types being mapped
    MAIR_EL1.write(
      // Attribute 1: Cacheable normal DRAM
      MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
      +
      MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
      +
      // Attribute 0: Device
      MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck
    );
  }

  /// Configure various settings for stage 1 of the EL1 translation regime
  fn configure_translation_control(&self) {
    let t0sz = (64 - bsp::memory::mmu::KernelAddrSpace::SIZE_SHIFT) as u64;

    TCR_EL1.write(
      TCR_EL1::TBI0::Used
      +
      TCR_EL1::IPS::Bits_40
      +
      TCR_EL1::TG0::KiB_64
      +
      TCR_EL1::SH0::Inner
      +
      TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
      +
      TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
      +
      TCR_EL1::A1::TTBR0
      +
      TCR_EL1::T0SZ.val(t0sz)
      +
      TCR_EL1::EPD1::DisableTTBR1Walks
    );
  }
}

/// Get a reference to the MMU instance
pub fn mmu() -> &'static impl memory::mmu::interface::MMU { &MMU }

use memory::mmu::MMUEnableError;

impl memory::mmu::interface::MMU for MemoryManagementUnit {
  unsafe fn enable_mmu_and_caching(&self) -> Result<(), MMUEnableError> {
    if unlikely(self.is_enabled()) { return Err(MMUEnableError::AlreadyEnabled); }

    // Fail early if translation granule is not supported
    if unlikely(!ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported)) {
      return Err(MMUEnableError::Other("Translation granule not supported by hardware"));
    }

    // Prepare the memory attribute indirection register
    self.set_up_mair();

    // Populate translation tables
    unsafe { KERNEL_TABLES.populate_tt_entries() }.map_err(MMUEnableError::Other)?;
    
    // Set the Translation Table Base Register
    TTBR0_EL1.set_baddr(unsafe { KERNEL_TABLES.phys_base_address() });

    self.configure_translation_control();

    // Force all previous changes to be seen before the MMU is enabled
    barrier::isb(barrier::SY);

    // Enable the MMU and turn on data and instruction caching
    SCTLR_EL1.modify(
      SCTLR_EL1::M::Enable
      +
      SCTLR_EL1::C::Cacheable
      +
      SCTLR_EL1::I::Cacheable
    );

    // Force the MMU init to complete before next instruction
    barrier::isb(barrier::SY);

    Ok(())
  }

  #[inline(always)]
  fn is_enabled(&self) -> bool { SCTLR_EL1.matches_all(SCTLR_EL1::M::Enable) }
}
