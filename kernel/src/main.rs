#![no_std]
#![no_main]

mod framebuffer;
mod printer;

use core::panic::PanicInfo;

use bootloader_api::BootInfo;

// use embedded_graphics::{
//     mono_font::{
//         ascii::FONT_10X20,
//         MonoTextStyle,
//     },
//     prelude::*,
//     text::{
//         Alignment,
//         Text,
//     },
// };

// use crate::framebuffer::Display;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        printer::set_framebuffer(framebuffer);

        // println!("Hello World{}", "!");

        kernel::init();

        // x86_64::instructions::interrupts::int3();

        // unsafe {
        //     *(0xdeadbeef as *mut u8) = 42;
        // };

        // let info = framebuffer.info();
        // let height = info.height;
        // let stride = info.stride;
        // let width = info.width;
        // let magnitude = 1.0 / (width as f32 / 255.0);
        // let area = stride as f32 * height as f32;
        //
        // for x in 0..width {
        //     for y in 0..height {
        //         let scale = magnitude * 255.0;
        //         let col = x as f32 / 255.0;
        //         let row = y as f32 / 255.0;
        //         let red = (col * scale) as u8;
        //         let green = (row * scale) as u8;
        //         let depth = y as f32 * stride as f32;
        //         let blue = (((depth + x as f32) / area) * 255.0) as u8;
        //
        //         framebuffer::set_pixel_in(
        //             framebuffer,
        //             framebuffer::Position { x, y },
        //             framebuffer::Color { red, green, blue },
        //         );
        //     }
        // }
    }

    kernel::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
