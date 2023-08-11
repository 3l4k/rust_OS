//writer
mod constants;

use core::{
    fmt::{self, Write},
    ptr,
};

use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use constants::font_constants;
use constants::font_constants::{BACKUP_CHAR, CHAR_RASTER_HEIGHT, FONT_WEIGHT};
// use core::ptr::NonNull;
use noto_sans_mono_bitmap::{get_raster, RasterizedChar};
// use crate::FRAMEBUFFER_WRITER;

/// Additional vertical space between lines
const LINE_SPACING: usize = 2;

/// Additional horizontal space between characters.
const LETTER_SPACING: usize = 0;

/// Padding from the border. Prevent that font is too close to border.
const BORDER_PADDING: usize = 1;

// static mut FRAMEBUFFER_BUFFER: Option<NonNull<[u8]>> = None;
// static mut FRAMEBUFFER_WRITER: Option<NonNull<FrameBufferWriter>> = None;

/// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(c, FONT_WEIGHT, CHAR_RASTER_HEIGHT)
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}

//conveting rgb to brg
fn swap_elements(matrix: &[[u8; 4]; 9]) -> [[u8; 4]; 9] {
    let mut swapped_matrix = [[0; 4]; 9];

    for (i, row) in matrix.iter().enumerate() {
        swapped_matrix[i][0] = row[2];
        swapped_matrix[i][1] = row[1];
        swapped_matrix[i][2] = row[0];
        swapped_matrix[i][3] = row[3];
    }

    swapped_matrix
}

/// Allows logging text to a pixel-based framebuffer.
pub struct FrameBufferWriter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
    x: usize,
    y: usize,
    color: usize,
}

pub mod my_macros {
    #[macro_export]
    macro_rules! print {
        ($($stmt:tt)*) => {
            {
                use crate::FRAMEBUFFER_WRITER;
                use core::fmt::write;
                let frame_buffer_writer = unsafe {FRAMEBUFFER_WRITER.as_mut().unwrap().as_mut() };

                write!(frame_buffer_writer, $($stmt)*).unwrap();
            }
        };
    }

    #[macro_export]
    macro_rules! println {
        ($($stmt:tt)*) => {
            {
                use crate::FRAMEBUFFER_WRITER;
                use core::fmt::write;
                let frame_buffer_writer = unsafe {FRAMEBUFFER_WRITER.as_mut().unwrap().as_mut() };

                writeln!(frame_buffer_writer, $($stmt)*).unwrap();
            }
        };
    }

    #[macro_export]
    macro_rules! input_char {
        () => {{
            use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
            use spin::Mutex;
            use x86_64::instructions::port::Port;

            lazy_static! {
                static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
                    Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
                );
            }

            let mut keyboard = KEYBOARD.lock();
            let mut port = Port::new(0x60);

            loop {
                let scancode: u8 = unsafe { port.read() };
                if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                    if let Some(key) = keyboard.process_keyevent(key_event) {
                        if let DecodedKey::Unicode(character) = key {
                            break character;
                        }
                    }
                }
            }
        }};
    }
}

impl FrameBufferWriter {
    /// Creates a new logger that uses the given framebuffer.

    /// Creates a new logger that uses the given framebuffer.
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut logger = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
            x: 0,
            y: 0,
            color: 0,
        };
        // logger.setChange(self.x, y);
        logger.clear();
        logger
    }

    fn newline(&mut self) {
        self.y_pos += font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }

    fn checkValidStartPosition(&mut self, x: usize, y: usize) -> bool {
        if ((x > self.info.width) || (y > self.info.height)) {
            false
        } else {
            true
        }
    }

    pub fn setChange(&mut self, x: usize, y: usize, color: usize) {
        if (self.checkValidStartPosition(x,y)) {
            self.x_pos = x;
            self.y_pos = y;
            self.color = color;

            //another way
            self.x = x;
            self.y = y;
        }
        // self.x_pos = x;
        // self.y_pos = y;
        // self.color = color;

        // //another way
        // self.x = x;
        // self.y = y;
        // self.clear();
    }

    /// Erases all text on the screen. Resets `self.x_pos` and `self.y_pos`.
    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;

        //another way
        // self.x_pos = self.x;
        // self.y_pos = self.y;
        self.framebuffer.fill(0);
    }

    /// Writes a single char to the framebuffer. Takes care of special control characters, such as
    /// newlines and carriage returns.
    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            '\x08' => self.backspace(),
            c => {
                let new_xpos = self.x_pos + font_constants::CHAR_RASTER_WIDTH;
                if new_xpos >= self.width() {
                    self.newline();
                }
                let new_ypos =
                    self.y_pos + font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.height() {
                    self.clear();
                }
                self.write_rendered_char(get_char_raster(c));
            }
        }
    }

    /// Prints a rendered char into the framebuffer.
    /// Updates `self.x_pos`.
    fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(self.x_pos + x, self.y_pos + y, *byte);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let mut colors_Rgb: [[u8; 4]; 9] = [
            [intensity, intensity, intensity, 0],              //white
            [intensity, 0, 0, 0],                              //red
            [0, intensity, 0, 0],                              //green
            [0, 0, intensity, 0],                              //blue
            [0, intensity, intensity, 0],                      //cyan
            [intensity, 0, intensity, 0],                      //magenta
            [intensity, intensity, 0, 0],                      //yellow
            [intensity, (intensity as f32 / 1.5) as u8, 0, 0], //orange
            [intensity / 2, 0, intensity / 2, 0],              //purple
        ];
        let colors_Bgr = swap_elements(&mut colors_Rgb);

        let pixel_offset = y * self.info.stride + x;
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => colors_Rgb[self.color],
            // PixelFormat::Bgr => [colorsRgb[0][2],colorsRgb[0][1],colorsRgb[0][0],0],
            PixelFormat::Bgr => colors_Bgr[self.color],
            PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
            other => {
                // set a supported (but invalid) pixel format before panicking to avoid a double
                // panic; it might not be readable though
                self.info.pixel_format = PixelFormat::Rgb;
                panic!("pixel format {:?} not supported in logger", other)
            }
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }

    /// Performs a backspace, deleting the last character printed on the screen.
    pub fn backspace(&mut self) {
        for i in 0..3 {
            if (self.x_pos == self.x + i) {
                self.x = self.x_pos;
                break;
            }
        }
        if ((self.x_pos == self.x) && (self.y_pos == self.y)) {
            self.x_pos = self.x;
            self.y_pos = self.y;
        } else {
            if self.x_pos >= BORDER_PADDING + font_constants::CHAR_RASTER_WIDTH {
                // self.x_pos -= font_constants::CHAR_RASTER_WIDTH;
                // Move the cursor back one character width
                self.x_pos -= font_constants::CHAR_RASTER_WIDTH;

                // Clear the last character by overwriting it with spaces
                for y in self.y_pos..(self.y_pos + font_constants::CHAR_RASTER_HEIGHT.val()) {
                    for x in self.x_pos..(self.x_pos + font_constants::CHAR_RASTER_WIDTH) {
                        // Assuming write_pixel correctly writes the pixel data
                        self.write_pixel(x, y, 0);
                    }
                }
            } else if (self.y_pos != BORDER_PADDING) {
                self.y_pos -= font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
                self.x_pos = self.width();
            }
        }
        // else if ((self.x_pos!=BORDER_PADDING)&&(self.y_pos!=BORDER_PADDING)) {
        //     self.y_pos -= font_constants::CHAR_RASTER_HEIGHT.val();
        //     self.x_pos = self.width();
        // }
    }
}

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
