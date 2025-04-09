// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! Architectural timer primitives
//! 
//! # Overview
//! 
//! Since arch modules are imported into generic modules using the path attribute,
//! the path of this file is: `crate::time::arch_time`

use core::{
  num:: {
    NonZeroU128,
    NonZeroU32,
    NonZeroU64,
  },
  ops::{
    Add,
    Div,
  },
  time::Duration,
};

use aarch64_cpu::{
  asm::barrier,
  registers::*,
};
use tock_registers::interfaces::Readable;

use crate::warn;

const NANOSEC_PER_SEC: NonZeroU64 = NonZeroU64::new(1_000_000_000).unwrap();

#[derive(Copy, Clone, PartialOrd, PartialEq)]
struct GenericTimerCounterValue(u64);

/// Boot assembly code overwrites this value with the value of CTNFRQ_EL0 before any Rust code is executed
/// This given value here is just a (safe) dummy/placeholder
#[unsafe(no_mangle)]
static ARCH_TIMER_COUNTER_FREQUENCY: NonZeroU32 = NonZeroU32::MIN;

fn arch_timer_counter_frequency() -> NonZeroU32 {
  // `read_volatile` is needed here to prevent the compiler from optimizing ARCH_TIMER_COUNTER_FREQUENCY away
  // This is safe because all the safety requirements as stated in `read_volatile`'s documentation are fulfilled
  unsafe { core::ptr::read_volatile(&ARCH_TIMER_COUNTER_FREQUENCY) } 
}

impl GenericTimerCounterValue {
  pub const MAX: Self = GenericTimerCounterValue(u64::MAX);
}

impl Add for GenericTimerCounterValue {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    GenericTimerCounterValue(self.0.wrapping_add(other.0))
  }
}

impl From<GenericTimerCounterValue> for Duration {
  fn from(counter_value: GenericTimerCounterValue) -> Self {
    if counter_value.0 == 0 { return Duration::ZERO; }

    let frequency: NonZeroU64 = arch_timer_counter_frequency().into();

    //Div<NonZeroU64> implementation for u64 cannot panic
    let secs = counter_value.0.div(frequency);

    // This is safe because frequency can never be greater than u32::MAX -
    // which means the largest theoretical value for `sub_second_counter_value` is `u32::MAX - 1`
    //
    // `sub_second_counter_value * NANOSEC_PER_SEC` cannot overflow a u64
    // The subsequent division ensures the result fits into a u32 since the max result is smaller than NANOSEC_PER_SEC
    // So casting to a u32 using `as` is cool
    let sub_second_counter_value = counter_value.0 % frequency;
    let nanos = unsafe { sub_second_counter_value.unchecked_mul(u64::from(NANOSEC_PER_SEC)).div(frequency) as u32 };

    Duration::new(secs, nanos)
  }
}

fn max_duration() -> Duration {
  Duration::from(GenericTimerCounterValue::MAX)
}

impl TryFrom<Duration> for GenericTimerCounterValue {
  type Error = &'static str;

  fn try_from(duration: Duration) -> Result<Self, Self::Error> {
    if duration < resolution() { return Ok(GenericTimerCounterValue(0)); }

    if duration > max_duration() { return Err("Conversion error; Duration too big"); }

    let frequency: u128 = u32::from(arch_timer_counter_frequency()) as u128;
    let duration: u128 = duration.as_nanos();

    // This is safe because frequency can never be greater than `u32::MAX` and `(Duration::MAX.as_nanos() * u32::MAX) < u128::MAX`
    let counter_value = unsafe { duration.unchecked_mul(frequency) }.div(NonZeroU128::from(NANOSEC_PER_SEC));

    // Since we checked above that we are <= max_duration() just cast to u64
    Ok(GenericTimerCounterValue(counter_value as u64))
  }
}

#[inline(always)]
fn read_cntpct() -> GenericTimerCounterValue {
  // Don't let the counter be read ahead of time due to out-of-order execution
  barrier::isb(barrier::SY);
  let cnt = CNTPCT_EL0.get();

  GenericTimerCounterValue(cnt)
}

/// The timer's resolution
pub fn resolution() -> Duration {
  Duration::from(GenericTimerCounterValue(1))
}

/// The uptime since device power-on
/// This includes time consumed by firmware and bootloaders
pub fn uptime() -> Duration {
  read_cntpct().into()
}

/// Spin for a given duration
pub fn spin_for(duration: Duration) {
  let curr_counter_value = read_cntpct();

  let counter_value_delta: GenericTimerCounterValue = match duration.try_into() {
    Err(msg) => {
      warn!("spin_for: {} - skipping", msg);
      return;
    }
    Ok(val) => val,
  };

  let counter_value_target = curr_counter_value + counter_value_delta;

  // Busy wait
  // Read CNTPCT_EL0 directly to avoid the ISB that is part of [`read_cntpct`]
  while GenericTimerCounterValue(CNTPCT_EL0.get()) < counter_value_target {}
}
