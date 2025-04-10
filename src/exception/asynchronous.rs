// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Asynchronous exception handling

#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/exception/asynchronous.rs"]
mod arch_asynchronous;

pub use arch_asynchronous::print_state;