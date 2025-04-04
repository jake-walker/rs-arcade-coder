#![no_std]
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Level, Output, OutputConfig, OutputPin},
    peripheral::Peripheral,
    spi::master::Spi,
    Blocking,
};

pub mod font;

type Color = (bool, bool, bool);

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
    channel_select_delay: Duration,
    latch_duration: Duration,
    display_buffer: [[u8; 9]; 6],
    anti_ghost: bool,
}

impl<'a> ArcadeCoder<'a> {
    pub fn new(
        display_spi: Spi<'a, Blocking>,
        display_a0: impl Peripheral<P = impl OutputPin> + 'a,
        display_a1: impl Peripheral<P = impl OutputPin> + 'a,
        display_a2: impl Peripheral<P = impl OutputPin> + 'a,
        display_oe: impl Peripheral<P = impl OutputPin> + 'a,
        display_latch: impl Peripheral<P = impl OutputPin> + 'a,
    ) -> Self {
        Self {
            display_spi,
            display_a0: Output::new(display_a0, Level::Low, OutputConfig::default()),
            display_a1: Output::new(display_a1, Level::Low, OutputConfig::default()),
            display_a2: Output::new(display_a2, Level::Low, OutputConfig::default()),
            display_oe: Output::new(display_oe, Level::High, OutputConfig::default()),
            display_latch: Output::new(display_latch, Level::Low, OutputConfig::default()),
            channel_select_delay: Duration::from_micros(10),
            latch_duration: Duration::from_micros(10),
            display_buffer: [[255; 9]; 6],
            anti_ghost: true,
        }
    }

    async fn set_channel(&mut self, channel: Option<usize>) -> () {
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

    pub fn clear_display(&mut self) {
        // clear the display by setting all bits on
        self.display_buffer = [[255; 9]; 6];
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> () {
        // if the coordinates are out of bounds, do nothing
        if x > 11 || y > 11 {
            return;
        }

        // calculate the byte and bit to be changed
        // the first value in the tuple is whether it is the first 4 pixels on either row
        // the second value is whether we want the top or the bottom row
        let (byte_idx, bit_idx) = match (x < 4, y < 6) {
            (true, true) => (3, 4 + x),
            (true, false) => (3, x),
            (false, true) => (0, x - 4),
            (false, false) => (6, x - 4),
        };

        // set the green, red and blue values respectively
        // each value is in the next byte so just need to add 1 and 2 to the byte index
        self.display_buffer[y % 6][byte_idx] = self.display_buffer[y % 6][byte_idx]
            & !(1 << bit_idx)
            | (u8::from(!color.1) << bit_idx);
        self.display_buffer[y % 6][byte_idx + 1] = self.display_buffer[y % 6][byte_idx + 1]
            & !(1 << bit_idx)
            | (u8::from(!color.0) << bit_idx);
        self.display_buffer[y % 6][byte_idx + 2] = self.display_buffer[y % 6][byte_idx + 2]
            & !(1 << bit_idx)
            | (u8::from(!color.2) << bit_idx);
    }

    fn draw_font_char(
        &mut self,
        n: usize,
        font: &[&[bool]],
        font_size: (usize, usize),
        start_x: usize,
        start_y: usize,
        color: Color,
    ) {
        let char_data = font[n];

        for row in 0..font_size.1 {
            for col in 0..font_size.0 {
                let pixel_on = char_data[row * font_size.0 + col];
                let x = start_x + col;
                let y = start_y + row;

                if pixel_on {
                    self.set_pixel(x, y, color);
                }
            }
        }
    }

    pub fn draw_digit(
        &mut self,
        n: u32,
        font: &[&[bool]],
        font_size: (usize, usize),
        start_x: usize,
        start_y: usize,
        color: Color,
    ) {
        self.draw_font_char(
            (n % 10).try_into().unwrap(),
            font,
            font_size,
            start_x,
            start_y,
            color,
        );
    }

    pub fn draw_char(
        &mut self,
        character: char,
        font: &[&[bool]],
        font_size: (usize, usize),
        start_x: usize,
        start_y: usize,
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

        self.draw_font_char(char_index, font, font_size, start_x, start_y, color);
    }

    async fn send_display_data(&mut self, words: &[u8]) -> () {
        self.display_oe.set_low();
        self.display_latch.set_low();

        self.display_spi
            .write(words)
            .expect("could not write display data");

        self.display_latch.set_high();
        Timer::after(self.latch_duration).await;
        self.display_latch.set_low();
        self.display_oe.set_high();
    }

    pub async fn draw(&mut self) -> () {
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
