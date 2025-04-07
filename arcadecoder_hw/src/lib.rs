//! Simple Rust library for working with the [Tech Will Save Us](https://en.wikipedia.org/wiki/Technology_Will_Save_Us) Arcade Coder
//!
//! Currently the display and single button presses work.

#![no_std]

use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Input, InputPin, Level, Output, OutputPin, Pull},
    peripheral::Peripheral,
    spi::master::Spi,
    Blocking,
};
use font::Font;

pub mod font;

/// 3-bit color
pub type Color = (bool, bool, bool);
/// Display coordinates
pub type Coordinates = (usize, usize);

pub const WHITE: Color = (true, true, true);
pub const YELLOW: Color = (true, true, false);
pub const CYAN: Color = (true, false, true);
pub const RED: Color = (true, false, false);
pub const MAGENTA: Color = (false, true, true);
pub const GREEN: Color = (false, true, false);
pub const BLUE: Color = (false, false, true);
pub const BLACK: Color = (false, false, false);

pub struct ArcadeCoder<'a> {
    display_spi: Spi<'a, Blocking>,
    display_a0: Output<'a>,
    display_a1: Output<'a>,
    display_a2: Output<'a>,
    display_oe: Output<'a>,
    display_latch: Output<'a>,
    inputs_1_7: Input<'a>,
    inputs_2_8: Input<'a>,
    inputs_3_9: Input<'a>,
    inputs_4_10: Input<'a>,
    inputs_5_11: Input<'a>,
    inputs_6_12: Input<'a>,
    channel_select_delay: Duration,
    latch_duration: Duration,
    display_buffer: [[u8; 9]; 6],
    anti_ghost: bool,
}

impl<'a> ArcadeCoder<'a> {
    /// Create a new instance of the Arcade Coder.
    ///
    /// # Example
    ///
    /// ```
    /// let p = esp_hal::init(esp_hal::Config::default());
    ///
    /// let timg0 = TimerGroup::new(p.TIMG0);
    /// esp_hal_embassy::init(timg0.timer0);
    ///
    /// let ac = ArcadeCoder::new(
    ///     Spi::new(
    ///         p.SPI2,
    ///         esp_hal::spi::master::Config::default()
    ///             .with_frequency(200_u32.kHz())
    ///             .with_mode(esp_hal::spi::Mode::_0)
    ///             .with_write_bit_order(esp_hal::spi::BitOrder::MsbFirst),
    ///     )
    ///     .expect("could not create spi")
    ///     .with_mosi(p.GPIO5)
    ///     .with_sck(p.GPIO17),
    ///     p.GPIO19,
    ///     p.GPIO18,
    ///     p.GPIO21,
    ///     p.GPIO4,
    ///     p.GPIO16,
    ///     p.GPIO39,
    ///     p.GPIO36,
    ///     p.GPIO35,
    ///     p.GPIO34,
    ///     p.GPIO33,
    ///     p.GPIO32,
    /// );
    /// ```
    pub fn new(
        display_spi: Spi<'a, Blocking>,
        display_a0: impl Peripheral<P = impl OutputPin> + 'a,
        display_a1: impl Peripheral<P = impl OutputPin> + 'a,
        display_a2: impl Peripheral<P = impl OutputPin> + 'a,
        display_oe: impl Peripheral<P = impl OutputPin> + 'a,
        display_latch: impl Peripheral<P = impl OutputPin> + 'a,
        inputs_1_7: impl Peripheral<P = impl InputPin> + 'a,
        inputs_2_8: impl Peripheral<P = impl InputPin> + 'a,
        inputs_3_9: impl Peripheral<P = impl InputPin> + 'a,
        inputs_4_10: impl Peripheral<P = impl InputPin> + 'a,
        inputs_5_11: impl Peripheral<P = impl InputPin> + 'a,
        inputs_6_12: impl Peripheral<P = impl InputPin> + 'a,
    ) -> Self {
        let input_pull = Pull::Up;

        Self {
            display_spi,
            display_a0: Output::new(display_a0, Level::Low),
            display_a1: Output::new(display_a1, Level::Low),
            display_a2: Output::new(display_a2, Level::Low),
            display_oe: Output::new(display_oe, Level::High),
            display_latch: Output::new(display_latch, Level::Low),
            inputs_1_7: Input::new(inputs_1_7, input_pull),
            inputs_2_8: Input::new(inputs_2_8, input_pull),
            inputs_3_9: Input::new(inputs_3_9, input_pull),
            inputs_4_10: Input::new(inputs_4_10, input_pull),
            inputs_5_11: Input::new(inputs_5_11, input_pull),
            inputs_6_12: Input::new(inputs_6_12, input_pull),
            channel_select_delay: Duration::from_micros(100),
            latch_duration: Duration::from_micros(50),
            display_buffer: [[255; 9]; 6],
            anti_ghost: true,
        }
    }

    async fn set_channel(&mut self, channel: Option<usize>) {
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

        self.display_a0.set_level(a0_level);
        self.display_a1.set_level(a1_level);
        self.display_a2.set_level(a2_level);
        // short delay for the output to stabilize
        Timer::after(self.channel_select_delay).await;
    }

    /// Clear the display buffer to make the screen blank.
    ///
    /// The `draw` method needs to be called after this to update the display.
    pub fn clear_display(&mut self) {
        // clear the display by setting all bits on
        self.display_buffer = [[255; 9]; 6];
    }

    fn get_display_indexes(&self, pos: Coordinates) -> (usize, usize) {
        match (pos.0 < 4, pos.1 < 6) {
            (true, true) => (3, 4 + pos.0),
            (true, false) => (3, pos.0),
            (false, true) => (0, pos.0 - 4),
            (false, false) => (6, pos.0 - 4),
        }
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

    async fn send_display_data(&mut self, words: &[u8]) {
        self.display_oe.set_low();
        self.display_latch.set_low();

        self.display_spi
            .write_bytes(words)
            .expect("could not write display data");

        self.display_latch.set_high();
        Timer::after(self.latch_duration).await;
        self.display_latch.set_low();
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

    fn read_row_values(&mut self) -> [bool; 6] {
        [
            self.inputs_1_7.is_high(),
            self.inputs_2_8.is_high(),
            self.inputs_3_9.is_high(),
            self.inputs_4_10.is_high(),
            self.inputs_5_11.is_high(),
            self.inputs_6_12.is_high(),
        ]
    }

    /// Get the coordinates of the currently pressed button (if any)
    ///
    /// Returns `None` if no button is pressed. _Indexing starts from 0, so (0, 0) is the top-left and (11, 11) is the bottom-right._
    pub async fn read_buttons(&mut self) -> Option<Coordinates> {
        self.display_latch.set_low();
        self.display_oe.set_low();

        for (i, v) in self.read_row_values().into_iter().enumerate() {
            if !v {
                continue;
            }

            let mut buf = [0xff; 9];

            for y in [i, i + 6] {
                for x in 0..12 {
                    let (byte_idx, bit_idx) = self.get_display_indexes((x, y));
                    buf[byte_idx + 1] &= !(1 << bit_idx); // set the red bit to 0

                    self.set_channel(None).await;
                    self.send_display_data(&buf).await;
                    let pressed = !self.read_row_values()[i];

                    self.send_display_data(&[0xff; 9]).await;

                    if pressed {
                        self.draw().await;
                        return Some((x, y));
                    }

                    buf[byte_idx + 1] |= 1 << bit_idx; // set the red bit to 1
                }
            }

            self.draw().await;
        }

        None
    }
}
