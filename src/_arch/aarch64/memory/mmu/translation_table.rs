// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Architectural virtual memory translation table
//! 
//! Only 64KiB granule is supported
//! 
//! # Overview
//! 
//! Since arch modules are imported into generic modules using the path attribute,
//! the path of this file is: `crate::memory::translation_table::arch_translation_table`

use core::convert;

use tock_registers::{
  interfaces::{
    Readable,
    Writeable,
  },
  register_bitfields,
  registers::InMemoryRegister,
};

use crate::{
  bsp,
  memory::mmu::{
    arch_mmu::{
      mair, Granule512MiB, Granule64KiB
    }, AccessPermissions, AttributeFields, MemAttributes
  }
};

// A level 2 table descriptor; as per ARMv8 Reference Manual section D4.4.1:
// "Descriptor encodings, ARMv8 level 0, level 1, and level 2 formats"
// https://developer.arm.com/documentation/ddi0487/la/?lang=en
register_bitfields! {
  u64,

  STAGE1_TABLE_DESCRIPTOR [
    /// Physical address of the next descriptor
    NEXT_LEVEL_TABLE_ADDR_64KiB OFFSET(16) NUMBITS(32) [], // [47:16]

    TYPE OFFSET(1) NUMBITS(1) [
      Block = 0,
      Table = 1
    ],

    VALID OFFSET(0) NUMBITS(1) [
      False = 0,
      True  = 1
    ]
  ]
}

// A level 3 page descriptor; as per ARMv8 Reference Manual section D4.4.3:
// "Memory attribute fields in the VMSAv8-64 translation table format descriptors"
// https://developer.arm.com/documentation/ddi0487/la/?lang=en
register_bitfields! {
  u64,

  STAGE1_PAGE_DESCRIPTOR [
    /// Unprivileged eXecute-Never
    UXN OFFSET(54) NUMBITS(1) [
      False = 0,
      True  = 1
    ],
  
    /// Privileged eXecute-Never
    PXN OFFSET(53) NUMBITS(1) [
      False = 0,
      True  = 1
    ],
  
    /// Physical address of the next table descriptor (lvl2) or the page descriptor (lvl3)
    OUTPUT_ADDR_64KiB OFFSET(16) NUMBITS(32) [], // [47:16]
  
    /// Access Flag
    AF OFFSET(10) NUMBITS(1) [
      False = 0,
      True  = 1
    ],
  
    /// SHareability field
    SH OFFSET(8) NUMBITS(2) [
      OuterShareable = 0b10,
      InnerShareable = 0b11
    ],
  
    /// Access Permissions
    AP OFFSET(6) NUMBITS(2) [
      RW_EL1     = 0b00,
      RO_EL1     = 0b10,
      RW_EL1_EL0 = 0b01,
      RO_EL1_EL0 = 0b11
    ],
  
    /// Memory attributes index into the MAIR_EL1 register
    AttrIndex OFFSET(2) NUMBITS(3) [],
  
    TYPE OFFSET(1) NUMBITS(1) [
      Reserved_Invalid = 0,
      Page             = 1
    ],
  
    VALID OFFSET(0) NUMBITS(1) [
      False = 0,
      True  = 1
    ]
  ]
}

/// A table descriptor for the 512MiB aperature
/// The output points to the next table
#[derive(Copy, Clone)]
#[repr(C)]
struct TableDescriptor {
  value: u64,
}

/// A page descriptor with a 64KiB aperature
/// The output points to physical memory
#[derive(Copy, Clone)]
#[repr(C)]
struct PageDescriptor {
  value: u64,
}

trait StartAddr {
  fn phys_start_addr_u64(&self) -> u64;
  fn phys_start_addr_usize(&self) -> usize;
}

const NUM_LVL2_TABLES: usize = bsp::memory::mmu::KernelAddrSpace::SIZE >> Granule512MiB::SHIFT;

/// Big monolithic struct for storing the translation tables
/// Individual levels must be 64 KiB alogned - so lvl3 is put first
#[repr(C)]
#[repr(align(65536))]
pub struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
  /// Page descriptors; each entry describes a 64 KiB memory window (aperature)
  lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

  /// Table descriptors; each describes a 512 MiB memory window (aperature)
  lvl2: [TableDescriptor; NUM_TABLES],
}

/// A translation table type for kernel space
pub type KernelTranslationTable = FixedSizeTranslationTable<NUM_LVL2_TABLES>;

// The binary is still identity mapped, so we don't need to convert here
impl<T, const N: usize> StartAddr for [T; N] {
  fn   phys_start_addr_u64(&self) -> u64   {self as *const T as u64   }
  fn phys_start_addr_usize(&self) -> usize {self as *const T as usize }
}

impl TableDescriptor {
  /// Create an instance
  /// Descriptor is invalid by default
  pub const fn new_zeroed() -> Self { Self { value: 0 } }

  /// Create an instance pointing to the specified address
  pub fn from_next_lvl_table_addr(phys_next_lvl_table_addr: usize) -> Self {
    let val = InMemoryRegister::<u64, STAGE1_TABLE_DESCRIPTOR::Register>::new(0);
    let shifted = phys_next_lvl_table_addr >> Granule64KiB::SHIFT;

    val.write(
      STAGE1_TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_64KiB.val(shifted as u64)
      +
      STAGE1_TABLE_DESCRIPTOR::TYPE::Table
      +
      STAGE1_TABLE_DESCRIPTOR::VALID::True
    );

    TableDescriptor { value: val.get() }
  }
}

/// Convert the kernel's generic memory attributes to hardware-specific attributes for the MMU
impl convert::From<AttributeFields> for tock_registers::fields::FieldValue<u64, STAGE1_PAGE_DESCRIPTOR::Register> {
  fn from(attribute_fields: AttributeFields) -> Self {
    // Memory attributes
    let mut desc = match attribute_fields.mem_attributes {
      MemAttributes::CacheableDRAM => {
        STAGE1_PAGE_DESCRIPTOR::SH::InnerShareable
        +
        STAGE1_PAGE_DESCRIPTOR::AttrIndex.val(mair::NORMAL)
      }
      MemAttributes::Device => {
        STAGE1_PAGE_DESCRIPTOR::SH::OuterShareable
        +
        STAGE1_PAGE_DESCRIPTOR::AttrIndex.val(mair::DEVICE)
      }
    };

    desc += match attribute_fields.acc_perms {
      AccessPermissions::ReadOnly  => STAGE1_PAGE_DESCRIPTOR::AP::RO_EL1,
      AccessPermissions::ReadWrite => STAGE1_PAGE_DESCRIPTOR::AP::RW_EL1,
    };

    // Access Permissions
    desc += if attribute_fields.execute_never {
      STAGE1_PAGE_DESCRIPTOR::PXN::True
    } else {
      STAGE1_PAGE_DESCRIPTOR::PXN::False
    };

    // Always set unprivileged execute-never as long as userspace is not implemented yet
    desc += STAGE1_PAGE_DESCRIPTOR::UXN::True;

    desc
  }
}

impl PageDescriptor {
  /// Create an instance
  /// Descriptor is invalid by default
  pub const fn new_zeroed() -> Self { Self { value: 0} }

  /// Create an instance
  pub fn from_output_addr(phys_output_addr: usize, attribute_fields: &AttributeFields) -> Self {
    let val = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(0);
    let shifted = phys_output_addr as u64 >> Granule64KiB::SHIFT;

    val.write(
      STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB.val(shifted)
      +
      STAGE1_PAGE_DESCRIPTOR::AF::True
      +
      STAGE1_PAGE_DESCRIPTOR::TYPE::Page
      +
      STAGE1_PAGE_DESCRIPTOR::VALID::True
      +
      (*attribute_fields).into()
    );

    Self { value: val.get() }
  }
}

impl<const NUM_TABLES: usize> FixedSizeTranslationTable<NUM_TABLES> {
  /// Create an instance
  pub const fn new() -> Self {
    // Can't have a zero-sized address space
    assert!(NUM_TABLES > 0);

    Self {
      lvl3: [[PageDescriptor::new_zeroed(); 8192]; NUM_TABLES],
      lvl2: [TableDescriptor::new_zeroed(); NUM_TABLES],
    }
  }

  /// Iterates over all static translation table entries and fills them
  /// 
  /// # Safety
  /// 
  /// Modifies a `static mut`; ensure it only happens from here
  pub unsafe fn populate_tt_entries(&mut self) -> Result<(), &'static str> {
    for (l2_nr, l2_entry) in self.lvl2.iter_mut().enumerate() {
      *l2_entry = TableDescriptor::from_next_lvl_table_addr(self.lvl3[l2_nr].phys_start_addr_usize());

      for (l3_nr, l3_entry) in self.lvl3[l2_nr].iter_mut().enumerate() {
        let virt_addr =
          (l2_nr << Granule512MiB::SHIFT)
          +
          (l3_nr << Granule64KiB::SHIFT);
        
        let (phys_output_addr, attribute_fields) = bsp::memory::mmu::virt_mem_layout().virt_addr_properties(virt_addr)?;

        *l3_entry = PageDescriptor::from_output_addr(phys_output_addr, &attribute_fields);
      }
    }

    Ok(())
  }

  /// The translation table's base address to be used for programming the MMU
  pub fn phys_base_address(&self) -> u64 { self.lvl2.phys_start_addr_u64() }
}
