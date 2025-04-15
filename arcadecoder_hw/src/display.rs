use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Level, Output, OutputPin},
    peripheral::Peripheral,
    peripherals::SPI2,
    spi::master::Spi,
    time::RateExtU32,
    Blocking,
};

use crate::{font::Font, Coordinates};

/// 3-bit color
pub type Color = (bool, bool, bool);

pub const WHITE: Color = (true, true, true);
pub const YELLOW: Color = (true, true, false);
pub const CYAN: Color = (false, true, true);
pub const RED: Color = (true, false, false);
pub const MAGENTA: Color = (true, false, true);
pub const GREEN: Color = (false, true, false);
pub const BLUE: Color = (false, false, true);
pub const BLACK: Color = (false, false, false);

pub struct ArcadeCoderDisplay<'a> {
    spi: Spi<'a, Blocking>,
    pin_a0: Output<'a>,
    pin_a1: Output<'a>,
    pin_a2: Output<'a>,
    pin_oe: Output<'a>,
    pin_latch: Output<'a>,

    /// The time to wait after switching channels for inputs to settle.
    pub channel_select_delay: Duration,

    /// The time to wait after latching.
    pub latch_delay: Duration,

    /// The current display buffer.
    pub display_buffer: [[u8; 9]; 6],

    /// Whether to draw a row blank after drawing to reduce ghosting. This comes at the cost of longer draw times.
    pub anti_ghost: bool,
}

impl<'a> ArcadeCoderDisplay<'a> {
    /// Create a new Arcade Coder display driver.
    ///
    /// # Example
    ///
    /// ```
    /// let p = esp_hal::init(esp_hal::Config::default());
    ///
    /// let mut display = ArcadeCoderDisplay::new(
    ///     p.SPI2, p.GPIO19, p.GPIO18, p.GPIO21, p.GPIO4, p.GPIO16, p.GPIO5, p.GPIO17,
    /// );
    /// ```
    pub fn new(
        spi_bus: SPI2,
        pin_a0: impl Peripheral<P = impl OutputPin> + 'a,
        pin_a1: impl Peripheral<P = impl OutputPin> + 'a,
        pin_a2: impl Peripheral<P = impl OutputPin> + 'a,
        pin_oe: impl Peripheral<P = impl OutputPin> + 'a,
        pin_latch: impl Peripheral<P = impl OutputPin> + 'a,
        pin_data: impl Peripheral<P = impl OutputPin> + 'a,
        pin_clock: impl Peripheral<P = impl OutputPin> + 'a,
    ) -> Self {
        Self {
            spi: Spi::new(
                spi_bus,
                esp_hal::spi::master::Config::default()
                    .with_frequency(200_u32.kHz())
                    .with_mode(esp_hal::spi::Mode::_0)
                    .with_write_bit_order(esp_hal::spi::BitOrder::MsbFirst),
            )
            .expect("could not create spi")
            .with_mosi(pin_data)
            .with_sck(pin_clock),
            pin_a0: Output::new(pin_a0, Level::Low),
            pin_a1: Output::new(pin_a1, Level::Low),
            pin_a2: Output::new(pin_a2, Level::Low),
            pin_oe: Output::new(pin_oe, Level::High),
            pin_latch: Output::new(pin_latch, Level::Low),
            channel_select_delay: Duration::from_micros(100),
            latch_delay: Duration::from_micros(50),
            display_buffer: [[255; 9]; 6],
            anti_ghost: true,
        }
    }

    pub(crate) async fn set_channel(&mut self, channel: Option<usize>) {
        let (a0_level, a1_level, a2_level) = match channel {
            Some(0) => (Level::Low, Level::High, Level::Low),
            Some(1) => (Level::High, Level::High, Level::Low),
            Some(2) => (Level::High, Level::Low, Level::High),
            Some(3) => (Level::Low, Level::Low, Level::High),
            Some(4) => (Level::High, Level::Low, Level::Low),
            Some(5) => (Level::Low, Level::High, Level::High),
            Some(7) => (Level::High, Level::High, Level::High),
            _ => (Level::Low, Level::Low, Level::Low),
        };

        self.pin_a0.set_level(a0_level);
        self.pin_a1.set_level(a1_level);
        self.pin_a2.set_level(a2_level);
        // short delay for the output to stabilize
        Timer::after(self.channel_select_delay).await;
    }

    /// Clear the display buffer to make the screen blank.
    ///
    /// The [`draw`] method needs to be called after this to update the display.
    ///
    /// [`draw`]: #method.draw
    pub fn clear(&mut self) {
        // clear the display by setting all bits on
        self.display_buffer = [[255; 9]; 6];
    }

    /// Set a pixel to a color
    ///
    /// _Indexing starts from 0, so (0, 0) is the top-left and (11, 11) is the bottom-right._
    pub fn set_pixel(&mut self, pos: Coordinates, color: Color) {
        // if the coordinates are out of bounds, do nothing
        if pos.0 > 11 || pos.1 > 11 {
            return;
        }

        // calculate the byte and bit to be changed
        // the first value in the tuple is whether it is the first 4 pixels on either row
        // the second value is whether we want the top or the bottom row
        let (byte_idx, bit_idx) = self.get_display_indexes(pos);

        // set the green, red and blue values respectively
        // each value is in the next byte so just need to add 1 and 2 to the byte index
        self.display_buffer[pos.1 % 6][byte_idx] = self.display_buffer[pos.1 % 6][byte_idx]
            & !(1 << bit_idx)
            | (u8::from(!color.1) << bit_idx);
        self.display_buffer[pos.1 % 6][byte_idx + 1] = self.display_buffer[pos.1 % 6][byte_idx + 1]
            & !(1 << bit_idx)
            | (u8::from(!color.0) << bit_idx);
        self.display_buffer[pos.1 % 6][byte_idx + 2] = self.display_buffer[pos.1 % 6][byte_idx + 2]
            & !(1 << bit_idx)
            | (u8::from(!color.2) << bit_idx);
    }

    pub fn draw_rect(&mut self, pos1: Coordinates, pos2: Coordinates, color: Color) {
        for x in pos1.0..=pos2.0 {
            for y in pos1.1..=pos2.1 {
                self.set_pixel((x, y), color);
            }
        }
    }

    fn draw_font_char(
        &mut self,
        n: usize,
        font: Font,
        font_size: (usize, usize),
        start_pos: Coordinates,
        color: Color,
    ) {
        let char_data = font[n];

        for row in 0..font_size.1 {
            for col in 0..font_size.0 {
                let pixel_on = char_data[row * font_size.0 + col];
                let pos = (start_pos.0 + col, start_pos.1 + row);

                if pixel_on {
                    self.set_pixel(pos, color);
                }
            }
        }
    }

    /// Draw a digit from a font
    ///
    /// ## Example
    /// ```
    /// use arcadecoder_hw::{
    ///     font::{FONT_5X5, FONT_5X5_SIZE},
    ///     WHITE,
    /// };
    ///
    /// ac.draw_digit(0, FONT_5X5, FONT_5X5_SIZE, (6, 0), WHITE);
    /// ```
    pub fn draw_digit(
        &mut self,
        n: u32,
        font: &[&[bool]],
        font_size: (usize, usize),
        start_pos: Coordinates,
        color: Color,
    ) {
        self.draw_font_char(
            (n % 10).try_into().unwrap(),
            font,
            font_size,
            start_pos,
            color,
        );
    }

    /// Draw a character from a font
    ///
    /// ## Example
    /// ```
    /// use arcadecoder_hw::{
    ///     font::{FONT_5X5, FONT_5X5_SIZE},
    ///     WHITE,
    /// };
    ///
    /// ac.draw_char('0', FONT_5X5, FONT_5X5_SIZE, (6, 0), WHITE);
    /// ```
    pub fn draw_char(
        &mut self,
        character: char,
        font: &[&[bool]],
        font_size: (usize, usize),
        start_pos: Coordinates,
        color: Color,
    ) {
        let char_index = match character {
            '0'..='9' => (character as u8 - b'0') as usize,
            'A'..='Z' => (character as u8 - b'A' + 10) as usize,
            _ => return,
        };

        if char_index >= font.len() {
            return;
        }

        self.draw_font_char(char_index, font, font_size, start_pos, color);
    }

    pub(crate) async fn send_display_data(&mut self, words: &[u8]) {
        self.pin_oe.set_low();
        self.pin_latch.set_low();

        self.spi
            .write_bytes(words)
            .expect("could not write display data");

        self.pin_latch.set_high();
        Timer::after(self.latch_delay).await;
        self.pin_latch.set_low();
    }

    pub(crate) fn get_display_indexes(&self, pos: Coordinates) -> (usize, usize) {
        match (pos.0 < 4, pos.1 < 6) {
            (true, true) => (3, 4 + pos.0),
            (true, false) => (3, pos.0),
            (false, true) => (0, pos.0 - 4),
            (false, false) => (6, pos.0 - 4),
        }
    }

    /// Update the display with the current buffer
    ///
    /// _This needs to be called regularly as the image disappears after a short time._
    pub async fn draw(&mut self) {
        // loop over each set of rows
        for i in 0_usize..6_usize {
            // create a copy of the display buffer
            let buf = self.display_buffer[i];

            self.set_channel(Some(i)).await;
            self.send_display_data(&buf).await;

            if self.anti_ghost {
                self.send_display_data(&[255; 9]).await;
            }
        }

        self.set_channel(None).await;
    }
}
