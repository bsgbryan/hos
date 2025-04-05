// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! A panic handler that infinitely waits.

use core::panic::PanicInfo;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unimplemented!()
}
