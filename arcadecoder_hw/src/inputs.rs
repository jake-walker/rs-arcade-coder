use embassy_futures::{join::join_array, select::select_array};
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Input, InputPin, Pull},
    peripheral::Peripheral,
};

use crate::{display::ArcadeCoderDisplay, Coordinates};

const ROW_PULL: Pull = Pull::Up;

pub struct ArcadeCoderInputs<'a> {
    rows_1_7: Input<'a>,
    rows_2_8: Input<'a>,
    rows_3_9: Input<'a>,
    rows_4_10: Input<'a>,
    rows_5_11: Input<'a>,
    rows_6_12: Input<'a>,

    pub debounce_delay: Duration,
}

impl<'a> ArcadeCoderInputs<'a> {
    /// Create a new Arcade Coder inputs driver.
    ///
    /// # Example
    ///
    /// ```
    /// let p = esp_hal::init(esp_hal::Config::default());
    ///
    /// let mut inputs = ArcadeCoderInputs::new(
    ///     p.GPIO39, p.GPIO36, p.GPIO35, p.GPIO34, p.GPIO33, p.GPIO32,
    /// );
    /// ```
    pub fn new(
        inputs_1_7: impl Peripheral<P = impl InputPin> + 'a,
        inputs_2_8: impl Peripheral<P = impl InputPin> + 'a,
        inputs_3_9: impl Peripheral<P = impl InputPin> + 'a,
        inputs_4_10: impl Peripheral<P = impl InputPin> + 'a,
        inputs_5_11: impl Peripheral<P = impl InputPin> + 'a,
        inputs_6_12: impl Peripheral<P = impl InputPin> + 'a,
    ) -> Self {
        Self {
            rows_1_7: Input::new(inputs_1_7, ROW_PULL),
            rows_2_8: Input::new(inputs_2_8, ROW_PULL),
            rows_3_9: Input::new(inputs_3_9, ROW_PULL),
            rows_4_10: Input::new(inputs_4_10, ROW_PULL),
            rows_5_11: Input::new(inputs_5_11, ROW_PULL),
            rows_6_12: Input::new(inputs_6_12, ROW_PULL),
            debounce_delay: Duration::from_millis(20),
        }
    }

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
    pub async fn read_buttons<'b>(
        &mut self,
        display: &mut ArcadeCoderDisplay<'b>,
    ) -> Option<Coordinates> {
        if let Some(row) = self.row_pressed() {
            let mut buf = [0xff; 9];

            for y in [row, row + 6] {
                for x in 0..12 {
                    let (byte_idx, bit_idx) = display.get_display_indexes((x, y));
                    buf[byte_idx + 1] &= !(1 << bit_idx); // set the red bit to 0

                    display.set_channel(None).await;
                    display.send_display_data(&buf).await;
                    let pressed = [
                        &self.rows_1_7,
                        &self.rows_2_8,
                        &self.rows_3_9,
                        &self.rows_4_10,
                        &self.rows_5_11,
                        &self.rows_6_12,
                    ][row]
                        .is_low();

                    display.send_display_data(&[0xff; 9]).await;

                    if pressed {
                        display.draw().await;
                        return Some((x, y));
                    }

                    buf[byte_idx + 1] |= 1 << bit_idx; // set the red bit to 1
                }
            }
        }

        None
    }
}
