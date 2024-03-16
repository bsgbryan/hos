#![no_std]
#![no_main]

use core::panic::PanicInfo;

use bootloader_api::BootInfo;

// use embedded_graphics::{
//     // mono_font::{
//     //     ascii::FONT_10X20,
//     //     MonoTextStyle,
//     // },
//     prelude::*,
//     // text::{
//     //     Alignment,
//     //     Text,
//     // },
// };

// use crate::framebuffer::Display;

// mod framebuffer;
mod printer;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
    //     let info = framebuffer.info();
    //     let height = info.height;
    //     let stride = info.stride;
    //     let width = info.width;
    //     let magnitude = 1.0 / (width as f32 / 255.0);
    //     let area = stride as f32 * height as f32;
    //
    //     for x in 0..width {
    //         for y in 0..height {
    //             let scale = magnitude * 255.0;
    //             let col = x as f32 / 255.0;
    //             let row = y as f32 / 255.0;
    //             let red = (col * scale) as u8;
    //             let green = (row * scale) as u8;
    //             let depth = y as f32 * stride as f32;
    //             let blue = (((depth + x as f32) / area) * 255.0) as u8;
    //
    //             framebuffer::set_pixel_in(
    //                 framebuffer,
    //                 framebuffer::Position { x, y },
    //                 framebuffer::Color { red, green, blue },
    //             );
    //         }
    //     }
    //     let mut display: Display = Display::new(framebuffer);
    //
    //     let character_style = MonoTextStyle::new(&FONT_10X20, RgbColor::WHITE);
    //
    //     let text = "Hello World, I'm HOS!";
    //     let x = 5;
    //     let y = display.size().height as i32 - (character_style.font.character_size.height as f32 * 0.5) as i32;
    //     let position = Point::new(x, y);
    //     Text::with_alignment(
    //         text,
    //         position,
    //         character_style,
    //         Alignment::Left,
    //     )
    //     .draw(&mut display).unwrap();
    // }

    println!("Hello World{}", "!");

    kernel::init();

    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // };

    // x86_64::instructions::interrupts::int3();

    kernel::hlt_loop();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
