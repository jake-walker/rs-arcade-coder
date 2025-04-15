//! Simple Rust library for working with the [Tech Will Save Us](https://en.wikipedia.org/wiki/Technology_Will_Save_Us) Arcade Coder.
//!
//! More projects, info and credits on Arcade Coder are [available here](https://github.com/padraigfl/awesome-arcade-coder), and hardware documentation is available [here](https://github.com/padraigfl/awesome-arcade-coder/wiki).
//!
//! Currently the display and single button presses work.

#![no_std]
#![doc = include_str!("../../docs/setup.md")]

use display::ArcadeCoderDisplay;
use esp_hal::{
    gpio::{InputPin, OutputPin},
    peripheral::Peripheral,
    peripherals::SPI2,
};
use inputs::ArcadeCoderInputs;

pub mod display;
pub mod font;
pub mod inputs;

/// Display coordinates
pub type Coordinates = (usize, usize);

pub struct ArcadeCoder<'a> {
    pub display: ArcadeCoderDisplay<'a>,
    pub inputs: ArcadeCoderInputs<'a>,
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
        spi_bus: SPI2,
        pin_a0: impl Peripheral<P = impl OutputPin> + 'a,
        pin_a1: impl Peripheral<P = impl OutputPin> + 'a,
        pin_a2: impl Peripheral<P = impl OutputPin> + 'a,
        pin_oe: impl Peripheral<P = impl OutputPin> + 'a,
        pin_latch: impl Peripheral<P = impl OutputPin> + 'a,
        pin_data: impl Peripheral<P = impl OutputPin> + 'a,
        pin_clock: impl Peripheral<P = impl OutputPin> + 'a,
        inputs_1_7: impl Peripheral<P = impl InputPin> + 'a,
        inputs_2_8: impl Peripheral<P = impl InputPin> + 'a,
        inputs_3_9: impl Peripheral<P = impl InputPin> + 'a,
        inputs_4_10: impl Peripheral<P = impl InputPin> + 'a,
        inputs_5_11: impl Peripheral<P = impl InputPin> + 'a,
        inputs_6_12: impl Peripheral<P = impl InputPin> + 'a,
    ) -> Self {
        Self {
            display: ArcadeCoderDisplay::new(
                spi_bus, pin_a0, pin_a1, pin_a2, pin_oe, pin_latch, pin_data, pin_clock,
            ),
            inputs: ArcadeCoderInputs::new(
                inputs_1_7,
                inputs_2_8,
                inputs_3_9,
                inputs_4_10,
                inputs_5_11,
                inputs_6_12,
            ),
        }
    }

    /// Split peripherals into separate structs.
    pub fn split(self) -> (ArcadeCoderDisplay<'a>, ArcadeCoderInputs<'a>) {
        (self.display, self.inputs)
    }
}
