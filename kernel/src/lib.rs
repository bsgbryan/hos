#![feature(abi_x86_interrupt)]
#![no_std]

// extern crate alloc;

pub mod framebuffer;
pub mod interrupts;
pub mod writer;

pub fn init() {
    interrupts::init_idt();
}