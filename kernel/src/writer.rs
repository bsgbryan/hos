// use core::fmt;
// use bootloader_api::info::{FrameBuffer};
// use embedded_graphics::geometry::Point;
// use embedded_graphics::mono_font::ascii::FONT_10X20;
// use embedded_graphics::mono_font::MonoTextStyle;
// use embedded_graphics::pixelcolor::RgbColor;
// use embedded_graphics::text::{Alignment, Text};
//
// use crate::framebuffer::Display;
//
// pub struct Writer<'a> {
//     buffer: &'a mut FrameBuffer
// }
//
// impl<'a> Writer<'a> {
//     pub fn new(&self, framebuffer: &'a mut FrameBuffer) -> Self {
//         Writer { buffer: framebuffer }
//     }
//     pub fn write_string(&mut self, s: &str) {
//         // let mut display = Display::new(self.buffer);
//         // let style = MonoTextStyle::new(&FONT_10X20, RgbColor::WHITE);
//         // let x = 5;
//         // let y = display.size().height as i32 - (style.font.character_size.height as f32 * 0.5) as i32;
//         // let position = Point::new(x, y);
//         //
//         // Text::with_alignment(
//         //     s,
//         //     position,
//         //     style,
//         //     Alignment::Left,
//         // )
//         // .draw(&mut display).unwrap();
//     }
// }
//
// impl<'a> fmt::Write for Writer<'a> {
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         self.write_string(s);
//         Ok(())
//     }
// }
//
//
