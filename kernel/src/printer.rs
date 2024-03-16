use core::fmt;
use bootloader_api::info::FrameBuffer;
use lazy_static::lazy_static;
use spin::Mutex;

use embedded_graphics::{
    mono_font::{
        ascii::FONT_10X20,
        MonoTextStyle,
    },
    prelude::*,
    text::{
        Alignment,
        Text,
    },
};

use crate::framebuffer::Display;

lazy_static! {
    /// A global `Writer` instance.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        framebuffer: None,
    });
}

pub fn set_framebuffer(buffer: &'static mut FrameBuffer) {
    let mut writer = WRITER.lock();
    writer.framebuffer = Option::from(buffer);
}

/// Supports newline characters and implements the `core::fmt::Write` trait.
pub struct Writer {
    framebuffer: Option<&'static mut FrameBuffer>,
}

impl Writer {
    fn write_string(&mut self, s: &str) {
        let buffer = self.framebuffer.as_mut().unwrap();
        let mut display: Display = Display::new(buffer);

        let character_style = MonoTextStyle::new(&FONT_10X20, RgbColor::WHITE);

        let x = 5;
        let y = display.size().height as i32 - (character_style.font.character_size.height as f32 * 0.5) as i32;
        let position = Point::new(x, y);

        Text::with_alignment(
            s,
            position,
            character_style,
            Alignment::Left,
        ).draw(&mut display).unwrap();
    }

    // /// Shifts all lines one line up and clears the last row.
    // fn new_line(&mut self) {
    //     /* TODO */
    // }
    //
    // /// Clears a row by overwriting it with blank characters.
    // fn clear_row(&mut self, row: usize) {
    //     /* TODO */
    // }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::printer::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text buffer through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}