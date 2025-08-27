//! Simple Rust library for working with the [Tech Will Save Us](https://en.wikipedia.org/wiki/Technology_Will_Save_Us) Arcade Coder.
//!
//! More projects, info and credits on Arcade Coder are [available here](https://github.com/padraigfl/awesome-arcade-coder), and hardware documentation is available [here](https://github.com/padraigfl/awesome-arcade-coder/wiki).
//!
//! Currently the display and single button presses work.

#![no_std]
#![doc = include_str!("../../docs/setup.md")]

use embassy_futures::{join::join_array, select::select_array};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Input, InputConfig, InputPin, Pull};
use esp_hal::{gpio::OutputPin, peripherals::SPI2};
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    spi::master::Spi,
    time::Rate,
    Blocking,
};

use crate::font::Font;

pub mod font;

/// Display coordinates
pub type Coordinates = (usize, usize);

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

pub struct ArcadeCoder<'a> {
    spi: Spi<'a, Blocking>,
    pin_a0: Output<'a>,
    pin_a1: Output<'a>,
    pin_a2: Output<'a>,
    pin_oe: Output<'a>,
    pin_latch: Output<'a>,
    rows_1_7: Input<'a>,
    rows_2_8: Input<'a>,
    rows_3_9: Input<'a>,
    rows_4_10: Input<'a>,
    rows_5_11: Input<'a>,
    rows_6_12: Input<'a>,

    /// The time to wait after switching channels for inputs to settle.
    pub channel_select_delay: Duration,

    /// The time to wait after latching.
    pub latch_delay: Duration,

    /// The current display buffer.
    pub display_buffer: [[u8; 9]; 6],

    /// Whether to draw a row blank after drawing to reduce ghosting. This comes at the cost of longer draw times.
    pub anti_ghost: bool,

    pub debounce_delay: Duration,
}

impl<'a> ArcadeCoder<'a> {
    /// Create a new instance of the Arcade Coder.
    ///
    /// # Example
    ///
    /// ```
    /// let p = esp_hal::init(esp_hal::Config::default());
    ///
    /// let mut ac = ArcadeCoder::new(
    ///     p.SPI2, p.GPIO19, p.GPIO18, p.GPIO21, p.GPIO4, p.GPIO16, p.GPIO5, p.GPIO17, p.GPIO39,
    ///     p.GPIO36, p.GPIO35, p.GPIO34, p.GPIO33, p.GPIO32,
    /// );
    /// ```
    pub fn new(
        spi_bus: SPI2<'a>,
        pin_a0: impl OutputPin + 'a,
        pin_a1: impl OutputPin + 'a,
        pin_a2: impl OutputPin + 'a,
        pin_oe: impl OutputPin + 'a,
        pin_latch: impl OutputPin + 'a,
        pin_data: impl OutputPin + 'a,
        pin_clock: impl OutputPin + 'a,
        inputs_1_7: impl InputPin + 'a,
        inputs_2_8: impl InputPin + 'a,
        inputs_3_9: impl InputPin + 'a,
        inputs_4_10: impl InputPin + 'a,
        inputs_5_11: impl InputPin + 'a,
        inputs_6_12: impl InputPin + 'a,
    ) -> Self {
        let output_cfg: OutputConfig = OutputConfig::default();
        let input_cfg: InputConfig = InputConfig::default().with_pull(Pull::Up);

        Self {
            // Display
            spi: Spi::new(
                spi_bus,
                esp_hal::spi::master::Config::default()
                    .with_frequency(Rate::from_khz(200_u32))
                    .with_mode(esp_hal::spi::Mode::_0)
                    .with_write_bit_order(esp_hal::spi::BitOrder::MsbFirst),
            )
            .expect("could not create spi")
            .with_mosi(pin_data)
            .with_sck(pin_clock),
            pin_a0: Output::new(pin_a0, Level::Low, output_cfg),
            pin_a1: Output::new(pin_a1, Level::Low, output_cfg),
            pin_a2: Output::new(pin_a2, Level::Low, output_cfg),
            pin_oe: Output::new(pin_oe, Level::High, output_cfg),
            pin_latch: Output::new(pin_latch, Level::Low, output_cfg),
            channel_select_delay: Duration::from_micros(100),
            latch_delay: Duration::from_micros(50),
            display_buffer: [[255; 9]; 6],
            anti_ghost: true,

            // Input
            rows_1_7: Input::new(inputs_1_7, input_cfg),
            rows_2_8: Input::new(inputs_2_8, input_cfg),
            rows_3_9: Input::new(inputs_3_9, input_cfg),
            rows_4_10: Input::new(inputs_4_10, input_cfg),
            rows_5_11: Input::new(inputs_5_11, input_cfg),
            rows_6_12: Input::new(inputs_6_12, input_cfg),
            debounce_delay: Duration::from_millis(20),
        }
    }

    // MARK: - Display

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

        self.spi.write(words).expect("could not write display data");

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

    // MARK: - Inputs

    /// Wait until a button in any row has been pressed.
    pub async fn wait_for_row_press(&mut self) -> usize {
        // wait for one of the inputs to go high
        select_array([
            self.rows_1_7.wait_for_high(),
            self.rows_2_8.wait_for_high(),
            self.rows_3_9.wait_for_high(),
            self.rows_4_10.wait_for_high(),
            self.rows_5_11.wait_for_high(),
            self.rows_6_12.wait_for_high(),
        ])
        .await
        .1
    }

    pub async fn wait_for_row_press_debounced(&mut self) -> Option<usize> {
        let row = self.wait_for_row_press().await;
        Timer::after(self.debounce_delay).await;
        if self.row_pressed().is_some() {
            return Some(row);
        }

        None
    }

    pub fn row_pressed(&mut self) -> Option<usize> {
        if self.rows_1_7.is_high() {
            Some(0)
        } else if self.rows_2_8.is_high() {
            Some(1)
        } else if self.rows_3_9.is_high() {
            Some(2)
        } else if self.rows_4_10.is_high() {
            Some(3)
        } else if self.rows_5_11.is_high() {
            Some(4)
        } else if self.rows_6_12.is_high() {
            Some(5)
        } else {
            None
        }
    }

    /// Wait until buttons in all rows have been released.
    pub async fn wait_for_row_release(&mut self) {
        // wait for all of the inputs to go low
        join_array([
            self.rows_1_7.wait_for_low(),
            self.rows_2_8.wait_for_low(),
            self.rows_3_9.wait_for_low(),
            self.rows_4_10.wait_for_low(),
            self.rows_5_11.wait_for_low(),
            self.rows_6_12.wait_for_low(),
        ])
        .await;
    }

    /// Get the coordinates of the currently pressed button (if any)
    ///
    /// Returns `None` if no button is pressed. _Indexing starts from 0, so (0, 0) is the top-left and (11, 11) is the bottom-right._
    pub async fn read_buttons(&mut self) -> Option<Coordinates> {
        if let Some(row) = self.row_pressed() {
            let mut buf = [0xff; 9];

            for y in [row, row + 6] {
                for x in 0..12 {
                    let (byte_idx, bit_idx) = self.get_display_indexes((x, y));
                    buf[byte_idx + 1] &= !(1 << bit_idx); // set the red bit to 0

                    self.set_channel(None).await;
                    self.send_display_data(&buf).await;
                    let pressed = [
                        &self.rows_1_7,
                        &self.rows_2_8,
                        &self.rows_3_9,
                        &self.rows_4_10,
                        &self.rows_5_11,
                        &self.rows_6_12,
                    ][row]
                        .is_low();

                    self.send_display_data(&[0xff; 9]).await;

                    if pressed {
                        self.draw().await;
                        return Some((x, y));
                    }

                    buf[byte_idx + 1] |= 1 << bit_idx; // set the red bit to 1
                }
            }
        }

        None
    }
}
