// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! General purpose code

/// Convert a size into a human-readable format
pub const fn size_human_readable_ceil(size: usize) -> (usize, &'static str) {
  const KIB: usize = 1024;
  const MIB: usize = KIB * 1024;
  const GIB: usize = MIB * 1024;

  if      (size / GIB) > 0 { (size.div_ceil(GIB), "GiB" ) }
  else if (size / MIB) > 0 { (size.div_ceil(GIB), "MiB" ) }
  else if (size / KIB) > 0 { (size.div_ceil(GIB), "KiB" ) }
  else                     { (size,               "Byte") }
}