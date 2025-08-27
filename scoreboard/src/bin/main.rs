#![no_std]
#![no_main]

use arcadecoder_hw::{
    font::{FONT_5X5, FONT_5X5_SIZE},
    ArcadeCoder, Color, GREEN, MAGENTA, RED, WHITE,
};
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    timer::timg::TimerGroup,
};
use esp_println::println;

const A_COLOR: Color = GREEN;
const B_COLOR: Color = MAGENTA;

esp_bootloader_esp_idf::esp_app_desc!();

#[derive(Debug)]
struct State {
    pub score_a: u8,
    pub score_b: u8,
    pub win_threshold: u8,
    pub win_diff: u8,

    a_winner: bool,
    b_winner: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            score_a: 0,
            score_b: 0,
            win_threshold: 11,
            win_diff: 2,
            a_winner: false,
            b_winner: false,
        }
    }

    fn check_win(&mut self) {
        let diff = self.score_a.abs_diff(self.score_b);

        self.a_winner = self.score_a > self.score_b
            && self.score_a >= self.win_threshold
            && diff >= self.win_diff;
        self.b_winner = self.score_b > self.score_a
            && self.score_b >= self.win_threshold
            && diff >= self.win_diff;
    }

    pub fn reset(&mut self) {
        self.score_a = 0;
        self.score_b = 0;
        self.check_win();
    }

    pub fn inc_score_a(&mut self) {
        self.score_a = self.score_a.checked_add(1).unwrap_or(self.score_a);
        self.check_win();
    }

    pub fn inc_score_b(&mut self) {
        self.score_b = self.score_b.checked_add(1).unwrap_or(self.score_b);
        self.check_win();
    }

    pub fn dec_score_a(&mut self) {
        self.score_a = self.score_a.checked_sub(1).unwrap_or(self.score_a);
        self.check_win();
    }

    pub fn dec_score_b(&mut self) {
        self.score_b = self.score_b.checked_sub(1).unwrap_or(self.score_b);
        self.check_win();
    }

    pub fn update_display(&self, ac: &mut ArcadeCoder<'_>) {
        ac.clear();

        let mut a_text_color = A_COLOR;
        let mut b_text_color = B_COLOR;

        if self.a_winner {
            ac.draw_rect((0, 0), (11, 11), A_COLOR);
            a_text_color = WHITE;
        } else if self.b_winner {
            ac.draw_rect((0, 0), (11, 11), B_COLOR);
            b_text_color = WHITE;
        }

        ac.draw_digit(
            (self.score_a % 10).into(),
            FONT_5X5,
            FONT_5X5_SIZE,
            (0, 6),
            a_text_color,
        );
        ac.draw_digit(
            ((self.score_a / 10) % 10).into(),
            FONT_5X5,
            FONT_5X5_SIZE,
            (0, 0),
            a_text_color,
        );
        ac.draw_digit(
            (self.score_b % 10).into(),
            FONT_5X5,
            FONT_5X5_SIZE,
            (7, 6),
            b_text_color,
        );
        ac.draw_digit(
            ((self.score_b / 10) % 10).into(),
            FONT_5X5,
            FONT_5X5_SIZE,
            (7, 0),
            b_text_color,
        );

        if self.win_threshold == 21 {
            ac.set_pixel((5, 11), RED);
            ac.set_pixel((6, 11), RED);
        }
    }
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    println!("Init!");
    let p = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let mut ac = ArcadeCoder::new(
        p.SPI2, p.GPIO19, p.GPIO18, p.GPIO21, p.GPIO4, p.GPIO16, p.GPIO5, p.GPIO17, p.GPIO39,
        p.GPIO36, p.GPIO35, p.GPIO34, p.GPIO33, p.GPIO32,
    );

    let mut state = State::new();

    state.update_display(&mut ac);

    let mut led = Output::new(p.GPIO22, Level::Low, OutputConfig::default());
    led.set_high();

    loop {
        match select(ac.wait_for_row_press(), Timer::after_millis(1)).await {
            // if the button was pressed, update the display and draw
            Either::First(_) => {
                Timer::after_millis(20).await;
                if ac.row_pressed().is_none() {
                    continue;
                }

                if let Some(coords) = ac.read_buttons().await {
                    if coords.1 == 11 && (coords.0 == 5 || coords.0 == 6) {
                        if state.win_threshold == 11 {
                            state.win_threshold = 21
                        } else {
                            state.win_threshold = 11
                        }
                        state.check_win();
                    } else if coords.1 == 11 {
                        // if the bottom row, reset the scores
                        state.reset();
                    } else if coords.0 < 6 && coords.1 < 6 {
                        // if top-left, add 1 to score a
                        state.inc_score_a();
                    } else if coords.0 < 6 {
                        // if bottom-left, subtract 1 from score a
                        state.dec_score_a();
                    } else if coords.1 < 6 {
                        // if top-right, add 1 to score b
                        state.inc_score_b();
                    } else {
                        // if bottom-right, subtract 1 from score b
                        state.dec_score_b();
                    }
                }

                ac.wait_for_row_release().await;
                state.update_display(&mut ac);
                ac.draw().await;
            }
            // ...otherwise just draw
            Either::Second(_) => {
                ac.draw().await;
            }
        }
    }
}
