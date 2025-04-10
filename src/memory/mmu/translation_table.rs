// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! AArch64 Translation Table

#[cfg(target_arch = "aarch64")]
#[path = "../../_arch/aarch64/memory/mmu/translation_table.rs"]
mod arch_translation_table;

pub use arch_translation_table::KernelTranslationTable;