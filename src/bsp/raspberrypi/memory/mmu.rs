// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! RPi Memory Management Unit

use core::ops::RangeInclusive;

use super::map as memory_map;
use crate::memory::mmu::*;

/// The kernel's address space defined by this BSP
pub type KernelAddrSpace = AddressSpace<{ memory_map::END_INCLUSIVE + 1 }>;

const NUM_MEM_RANGES: usize = 2;

/// The virtual memory layout
/// The layout must contain only special ranges - meaning only things _not_ normal cacheable DRAM
/// It is agnostic of the paging granularity that the architecture's MMU will use
pub static LAYOUT: KernelVirtualLayout<NUM_MEM_RANGES> = KernelVirtualLayout::new(
  memory_map::END_INCLUSIVE,
  [
    TranslationDescriptor {
      name: "Kernel code and RO data",
      virtual_range: code_range_inclusive,
      physical_range_translation: Translation::Identity,
      attribute_fields: AttributeFields {
        mem_attributes: MemAttributes::CacheableDRAM,
        acc_perms: AccessPermissions::ReadOnly,
        execute_never: false,
      },
    },
    TranslationDescriptor {
      name: "Primary Device MMIO",
      virtual_range: mmio_range_inclusive,
      physical_range_translation: Translation::Identity,
      attribute_fields: AttributeFields {
        mem_attributes: MemAttributes::Device,
        acc_perms: AccessPermissions::ReadWrite,
        execute_never: true,
      },
    },
  ]
);

fn code_range_inclusive() -> RangeInclusive<usize> {
  // Notice the subtraction to turn the exclusive end into an inclusive end
  RangeInclusive::new(super::code_start(), super::code_end_exclusive() - 1)
}

fn mmio_range_inclusive() -> RangeInclusive<usize> {
  RangeInclusive::new(memory_map::mmio::START, memory_map::mmio::END_INCLUSIVE)
}

/// Get a reference to the virtual memory layout
pub fn virt_mem_layout() -> &'static KernelVirtualLayout<NUM_MEM_RANGES> { &LAYOUT }
