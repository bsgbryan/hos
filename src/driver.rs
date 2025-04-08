// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

//! OS driver support

use crate::{
  println,
  synchronization::{
    interface::Mutex,
    NullLock,
  },
};

const NUM_DRIVERS: usize = 5;

struct DriverManagerInner {
  next_index: usize,
  descriptors: [Option<DeviceDriverDescriptor>; NUM_DRIVERS],
}

pub mod interface {
  /// Driver interface
  pub trait DeviceDriver {
    /// Returns a string identifying driver
    fn compatible(&self) -> &'static str;
  
    /// Called by the kernel to initialize the device
    unsafe fn init(&self) -> Result<(), &'static str> {
      Ok(())
    }
  }
}

/// Optional callback invoked after device is successfully iniitalized
pub type DeviceDriverPostInitCallback = unsafe fn() -> Result<(), &'static str>;

/// Describes a device driver
#[derive(Copy, Clone)]
pub struct DeviceDriverDescriptor {
  device_driver: &'static (dyn interface::DeviceDriver + Sync),
  post_init_callback: Option<DeviceDriverPostInitCallback>,
}

/// Managed all device drivers
pub struct DriverManager {
  inner: NullLock<DriverManagerInner>,
}

unsafe impl Sync for DriverManager {}

static DRIVER_MANAGER: DriverManager = DriverManager::new();

impl DriverManagerInner {
  /// Creates an instance
  pub const fn new() -> Self {
    Self {
      next_index: 0,
      descriptors: [None; NUM_DRIVERS],
    }
  }
}

impl DeviceDriverDescriptor {
  /// Creates an instance
  pub fn new(
    device_driver: &'static (dyn interface::DeviceDriver + Sync),
    post_init_callback: Option<DeviceDriverPostInitCallback>,
  ) -> Self {
    Self {
      device_driver,
      post_init_callback,
    }
  }
}

/// Return a reference to the global driver manager
pub fn driver_manager() -> &'static DriverManager {
  &DRIVER_MANAGER
}

impl DriverManager {
  /// Create an instance
  pub const fn new() -> Self {
    Self {
      inner: NullLock::new(DriverManagerInner::new()),
    }
  }

  pub fn register_driver(&self, descriptor: DeviceDriverDescriptor) {
    self.inner.lock(|i| {
      i.descriptors[i.next_index] = Some(descriptor);
      i.next_index += 1;
    })
  }

  fn for_each_descriptor<'a>(&'a self, f: impl FnMut(&'a DeviceDriverDescriptor)) {
    self.inner.lock(|i| {
      i.
        descriptors.
        iter().
        filter_map(|x| x.as_ref()).
        for_each(f)
    })
  }

  /// Fully initialize all drivers
  /// 
  /// # Safety
  /// 
  /// - During init, drivers might to things with system-wide impact
  pub fn init_drivers(&self) {
    self.for_each_descriptor(|d| {
      // 1. Initialize the driver
      if let Err(e) = unsafe { d.device_driver.init() } {
        panic!("Error initializing {} driver: {}", d.device_driver.compatible(), e);
      }

      // 2. Invoke post init callback - if one exxists
      if let Some(cb) = &d.post_init_callback {
        if let Err(e) = unsafe { cb() } {
          panic!("Error during {} driver post-init callback: {}", d.device_driver.compatible(), e);
        }
      }
    });
  }

  /// Enumerate all registered device drivers
  pub fn enumerate(&self) {
    let mut i: usize = 1;

    self.for_each_descriptor(|d| {
      println!("\t{}: {}", i, d.device_driver.compatible());
      i += 1;
    })
  }
}