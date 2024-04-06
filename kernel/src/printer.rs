use bootloader_api::info::{
    FrameBufferInfo,
    PixelFormat,
};

use core::{
    fmt,
    ptr,
};

use font_constants::BACKUP_CHAR;

use lazy_static::lazy_static;

use noto_sans_mono_bitmap::{
    get_raster,
    get_raster_width,
    FontWeight,
    RasterHeight,
    RasterizedChar,
};

use spin::Mutex;

/// Additional vertical space between lines
const LINE_SPACING: usize = 2;
/// Additional horizontal space between characters.
const LETTER_SPACING: usize = 0;

/// Padding from the border. Prevent that font is too close to border.
const BORDER_PADDING: usize = 1;

/// Constants for the usage of the [`noto_sans_mono_bitmap`] crate.
mod font_constants {
    use super::*;

    /// Height of each char raster. The font size is ~0.84% of this. Thus, this is the line height that
    /// enables multiple characters to be side-by-side and appear optically in one line in a natural way.
    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;

    /// The width of each single symbol of the mono space font.
    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);

    /// Backup character if a desired symbol is not available by the font.
    /// The '�' character requires the feature "unicode-specials".
    pub const BACKUP_CHAR: char = '�';

    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;
}

lazy_static! {
    /// A global `Writer` instance.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        framebuffer: None,
        info:        None,
        x: BORDER_PADDING,
        y: BORDER_PADDING,
    });
}

pub fn set_framebuffer(
    buffer: &'static mut [u8],
    info:   FrameBufferInfo,
) {
    let mut writer = WRITER.lock();
    writer.framebuffer = Option::from(buffer);
    writer.info        = Option::from(info);
    writer.clear();
}

/// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(
            c,
            font_constants::FONT_WEIGHT,
            font_constants::CHAR_RASTER_HEIGHT,
        )
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}

/// Supports newline characters and implements the `core::fmt::Write` trait.
pub struct Writer {
    framebuffer: Option<&'static mut [u8]>,
    info: Option<FrameBufferInfo>,
    x: usize,
    y: usize,
}

impl Writer {
    fn newline(&mut self) {
        self.y += font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x = BORDER_PADDING;
    }

    /// Erases all text on the screen. Resets `self.x` and `self.y`.
    pub fn clear(&mut self) {
        self.x = BORDER_PADDING;
        self.y = BORDER_PADDING;

        if let Some(buffer) = self.framebuffer.as_mut() {
            buffer.fill(0);
        }
    }

    fn width(&self) -> usize {
        self.info.unwrap().width
    }

    fn height(&self) -> usize {
        self.info.unwrap().height
    }

    /// Writes a single char to the framebuffer. Takes care of special control characters, such as
    /// newlines and carriage returns.
    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                let next_x = self.x + font_constants::CHAR_RASTER_WIDTH;
                if next_x >= self.width() {
                    self.newline();
                }
                let next_y = self.y + font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if next_y >= self.height() {
                    self.clear();
                }
                self.write_rendered_char(get_char_raster(c));
            }
        }
    }

    /// Prints a rendered char into the framebuffer.
    /// Updates `self.x`.
    fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(self.x + x, self.y + y, *byte);
            }
        }
        self.x += rendered_char.width() + LETTER_SPACING;
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.info.unwrap().stride + x;
        let color = match self.info.unwrap().pixel_format {
            PixelFormat::Rgb => [intensity, intensity, intensity / 2, 0],
            PixelFormat::Bgr => [intensity / 2, intensity, intensity, 0],
            PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
            other => {
                // set a supported (but invalid) pixel format before panicking to avoid a double
                // panic; it might not be readable though
                // if let Some(& mut info) = self.info {
                //     info.as_mut().pixel_format = PixelFormat::Rgb;
                // }
                panic!("pixel format {:?} not supported in logger", other)
            }
        };
        let bytes_per_pixel = self.info.unwrap().bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        if let Some(buffer) = self.framebuffer.as_mut() {
            buffer[byte_offset..(byte_offset + bytes_per_pixel)].
                copy_from_slice(&color[..bytes_per_pixel]);

            let _ = unsafe { ptr::read_volatile(&buffer[byte_offset]) };
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
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