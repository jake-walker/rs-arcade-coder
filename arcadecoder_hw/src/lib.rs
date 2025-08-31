//! Simple Rust library for working with the [Tech Will Save Us](https://en.wikipedia.org/wiki/Technology_Will_Save_Us) Arcade Coder.
//!
//! More projects, info and credits on Arcade Coder are [available here](https://github.com/padraigfl/awesome-arcade-coder), and hardware documentation is available [here](https://github.com/padraigfl/awesome-arcade-coder/wiki).
//!
//! Currently the display and single button presses work.

#![no_std]

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

#[derive(Clone, Copy, Debug)]
pub enum ButtonEvent {
    Pressed(u8, u8),
    Released(u8, u8),
}

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

    /// A matrix of button presses corresponding to the physical layout.
    pub button_presses: [[bool; 12]; 12],

    /// The time to wait after displaying a row on the display.
    pub channel_on_time: Duration,

    /// The number of reads required for a button press to register.
    pub debounce_reads: u8,

    prev_read: [[bool; 12]; 12],
    stable_count: [[u8; 12]; 12],
    stable_state: [[bool; 12]; 12],
}

impl<'a> ArcadeCoder<'a> {
    /// Create a new instance of the Arcade Coder.
    ///
    /// **Example:**
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
                    .with_frequency(Rate::from_mhz(8_u32))
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
            channel_select_delay: Duration::from_micros(5),
            latch_delay: Duration::from_micros(2),
            display_buffer: [[255; 9]; 6],
            channel_on_time: Duration::from_micros(1388),

            // Input
            rows_1_7: Input::new(inputs_1_7, input_cfg),
            rows_2_8: Input::new(inputs_2_8, input_cfg),
            rows_3_9: Input::new(inputs_3_9, input_cfg),
            rows_4_10: Input::new(inputs_4_10, input_cfg),
            rows_5_11: Input::new(inputs_5_11, input_cfg),
            rows_6_12: Input::new(inputs_6_12, input_cfg),
            button_presses: [[false; 12]; 12],
            debounce_reads: 5,
            prev_read: [[false; 12]; 12],
            stable_count: [[0u8; 12]; 12],
            stable_state: [[false; 12]; 12],
        }
    }

    // MARK: - Display

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

        self.pin_a0.set_level(a0_level);
        self.pin_a1.set_level(a1_level);
        self.pin_a2.set_level(a2_level);
        // short delay for the output to stabilize
        Timer::after(self.channel_select_delay).await;
    }

    /// Clear the display buffer to make the screen blank.
    ///
    /// The [`scan`] method needs to be called after this to update the display.
    ///
    /// [`scan`]: #method.scan
    pub fn clear(&mut self) {
        // clear the display by setting all bits on
        self.display_buffer = [[255; 9]; 6];
    }

    /// Set a pixel to a color
    ///
    /// _Indexing starts from 0, so (0, 0) is the top-left and (11, 11) is the bottom-right._
    ///
    /// The [`scan`] method needs to be called after this to update the display.
    ///
    /// [`scan`]: #method.scan
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

    async fn send_display_data(&mut self, words: &[u8]) {
        self.pin_oe.set_high();
        self.pin_latch.set_low();

        self.spi.write(words).expect("could not write display data");

        self.pin_latch.set_high();
        Timer::after(self.latch_delay).await;
        self.pin_latch.set_low();
        self.pin_oe.set_low();
    }

    fn get_display_indexes(&self, pos: Coordinates) -> (usize, usize) {
        match (pos.0 < 4, pos.1 < 6) {
            (true, true) => (3, 4 + pos.0),
            (true, false) => (3, pos.0),
            (false, true) => (0, pos.0 - 4),
            (false, false) => (6, pos.0 - 4),
        }
    }

    // MARK: - Inputs

    /// Handle button press events. This takes care of debouncing inputs and returns an event for button presses and releases.
    ///
    /// The handler function passed in to the function will be called for each button event.
    ///
    /// The [`scan`] method needs to be called at regular intervals for button presses to update.
    ///
    /// [`scan`]: #method.scan
    pub fn handle_input_events<F>(&mut self, mut handler: F)
    where
        F: FnMut(ButtonEvent),
    {
        for y in 0..12_usize {
            for x in 0..12_usize {
                let cur = self.button_presses[y][x];

                if cur == self.prev_read[y][x] {
                    self.stable_count[y][x] = self.stable_count[y][x].saturating_add(1);
                } else {
                    self.stable_count[y][x] = 0;
                    self.prev_read[y][x] = cur;
                }

                if self.stable_count[y][x] >= self.debounce_reads && cur != self.stable_state[y][x]
                {
                    self.stable_state[y][x] = cur;
                    if cur {
                        handler(ButtonEvent::Pressed(x as u8, y as u8));
                    } else {
                        handler(ButtonEvent::Released(x as u8, y as u8));
                    }
                }
            }
        }
    }

    /// Handle button press events. This takes care of debouncing inputs and returns an event for button presses and releases.
    ///
    /// This uses sends events from [`handle_input_events`] but instead sends events through to a channel, rather than a handler function.
    ///
    /// The [`scan`] method needs to be called at regular intervals for button presses to update.
    ///
    /// [`handle_input_events`]: #method.handle_input_events
    /// [`scan`]: #method.scan
    pub fn handle_input_events_to_channel<const N: usize>(
        &mut self,
        ch: &embassy_sync::channel::Channel<
            embassy_sync::blocking_mutex::raw::NoopRawMutex,
            ButtonEvent,
            N,
        >,
    ) {
        self.handle_input_events(|e| {
            let _ = ch.try_send(e);
        });
    }

    /// Update the display while also scanning for button inputs.
    ///
    /// This draws each row in turn and checks for button presses on the same row before going to the next row.
    ///
    /// This must be called at a regular interval to ensure the display is refreshed and button inputs are registered.
    pub async fn scan(&mut self) {
        // helpers to access inputs by index without repeating logic
        let read_input = |s: &mut ArcadeCoder<'a>, idx: usize| -> bool {
            match idx {
                0 => s.rows_1_7.is_low(),
                1 => s.rows_2_8.is_low(),
                2 => s.rows_3_9.is_low(),
                3 => s.rows_4_10.is_low(),
                4 => s.rows_5_11.is_low(),
                5 => s.rows_6_12.is_low(),
                _ => false,
            }
        };

        // clear previous button state for this pass
        for i in 0..12_usize {
            for j in 0..12_usize {
                self.button_presses[i][j] = false;
            }
        }

        // drive each channel and scan its 12 columns
        for channel in 0_usize..6_usize {
            // copy of the current rows buffer
            let buf = self.display_buffer[channel];
            // buffer for performing button tests
            let mut test_buf = [0xff; 9];

            // select this channel and show the normal frame first
            self.set_channel(Some(channel)).await;
            self.send_display_data(&buf).await;
            // wait a short duration
            Timer::after(self.channel_on_time).await;

            // select the input channel
            self.set_channel(None).await;

            // scan columns for button presses
            for x in 0..12_usize {
                for physical_row in [channel, channel + 6_usize] {
                    // get indexes corresponding to the column for the bits to be changed
                    let (byte_idx, bit_idx) = self.get_display_indexes((x, physical_row));

                    // for the input testing buffer, set the red bit to low
                    test_buf[byte_idx + 1] &= !(1 << bit_idx);

                    // send the test pattern
                    self.send_display_data(&test_buf).await;

                    // read the input line for this channel
                    let pressed = read_input(self, channel);

                    if pressed {
                        // mark the button as pressed
                        self.button_presses[physical_row][x] = true;
                    }

                    // unset the red bit for the next pass
                    test_buf[byte_idx + 1] |= 1 << bit_idx;
                }
            }

            // send a blank row to reduce ghosting
            self.send_display_data(&[255; 9]).await;
        }
    }
}
