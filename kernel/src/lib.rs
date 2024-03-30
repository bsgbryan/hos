#![no_std]

#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(type_alias_impl_trait)]

extern crate alloc;

pub mod allocator;
pub mod framebuffer;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod printer;

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