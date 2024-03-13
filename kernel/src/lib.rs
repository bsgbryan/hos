#![feature(abi_x86_interrupt)]
#![no_std]

// extern crate alloc;

pub mod framebuffer;
pub mod interrupts;
pub mod gdt;
pub mod writer;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe {
        interrupts::PICS.lock().initialize()
    };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}