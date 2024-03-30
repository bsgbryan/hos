#![no_std]
#![no_main]

extern crate alloc;

mod framebuffer;
mod printer;

use alloc::boxed::Box;

use core::panic::PanicInfo;

use bootloader_api::{
    BootInfo,
    config::{
        BootloaderConfig,
        Mapping,
    },
};


pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        use hos_kernel::{
            allocator,
            memory,
            memory::BootInfoFrameAllocator,
        };

        use x86_64::{
            structures::paging::Page,
            VirtAddr,
        };

        printer::set_framebuffer(framebuffer);

        hos_kernel::init();

        if let Some(addr) = boot_info.physical_memory_offset.take() {
            let phys_mem_offset = VirtAddr::new(addr);
            let mut mapper = unsafe { memory::init(phys_mem_offset) };
            let mut frame_allocator = unsafe {
                BootInfoFrameAllocator::init(&boot_info.memory_regions)
            };

            // map an unused page
            let page = Page::containing_address(VirtAddr::new(0));
            memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

            // write the string `New!` to the screen through the new mapping
            let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
            unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

            allocator::init_heap(&mut mapper, &mut frame_allocator).
                expect("heap initialization failed");

            let x = Box::new(41);

            println!("heap_value at {:p}", x);
            // println!("It worked!");
        }
    }

    hos_kernel::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
