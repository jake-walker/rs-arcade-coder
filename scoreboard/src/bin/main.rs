#![no_std]
#![no_main]

use arcadecoder_hw::{
    font::{FONT_5X5, FONT_5X5_SIZE},
    ArcadeCoder, ButtonEvent, Color, GREEN, MAGENTA, RED, WHITE,
};
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel, mutex::Mutex};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    timer::timg::TimerGroup,
};
use esp_println::println;

const A_COLOR: Color = GREEN;
const B_COLOR: Color = MAGENTA;

static mut EVENT_CH: Option<Channel<NoopRawMutex, ButtonEvent, 64>> = None;
static mut REDRAW_CH: Option<Channel<NoopRawMutex, (), 4>> = None;
static mut STATE_MUTEX: Option<Mutex<NoopRawMutex, State>> = None;

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

#[embassy_executor::task]
async fn state_task(
    ev: &'static Channel<NoopRawMutex, ButtonEvent, 64>,
    redraw: &'static Channel<NoopRawMutex, (), 4>,
    state_mutex: &'static Mutex<NoopRawMutex, State>,
) {
    loop {
        let event = ev.receive().await;

        {
            let mut s = state_mutex.lock().await;
            match event {
                ButtonEvent::Pressed(x, y) => {
                    if y == 11 && (x == 5 || x == 6) {
                        if s.win_threshold == 11 {
                            s.win_threshold = 21
                        } else {
                            s.win_threshold = 11
                        }
                        s.check_win();
                    } else if y == 11 {
                        // if the bottom row, reset the scores
                        s.reset();
                    } else if x < 6 && y < 6 {
                        // if top-left, add 1 to score a
                        s.inc_score_a();
                    } else if x < 6 {
                        // if bottom-left, subtract 1 from score a
                        s.dec_score_a();
                    } else if y < 6 {
                        // if top-right, add 1 to score b
                        s.inc_score_b();
                    } else {
                        // if bottom-right, subtract 1 from score b
                        s.dec_score_b();
                    }

                    let _ = redraw.try_send(());
                }
                ButtonEvent::Released(_, _) => {}
            }
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    println!("Init!");
    let p = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let mut ac = ArcadeCoder::new(
        p.SPI2, p.GPIO19, p.GPIO18, p.GPIO21, p.GPIO4, p.GPIO16, p.GPIO5, p.GPIO17, p.GPIO39,
        p.GPIO36, p.GPIO35, p.GPIO34, p.GPIO33, p.GPIO32,
    );

    unsafe {
        EVENT_CH = Some(Channel::new());
        REDRAW_CH = Some(Channel::new());
        STATE_MUTEX = Some(Mutex::new(State::new()));
    }

    let ev: &'static Channel<NoopRawMutex, ButtonEvent, 64> = unsafe { EVENT_CH.as_ref().unwrap() };
    let redraw: &'static Channel<NoopRawMutex, (), 4> = unsafe { REDRAW_CH.as_ref().unwrap() };
    let state_mutex: &'static Mutex<NoopRawMutex, State> = unsafe { STATE_MUTEX.as_ref().unwrap() };

    spawner.spawn(state_task(ev, redraw, state_mutex)).unwrap();

    let mut led = Output::new(p.GPIO22, Level::Low, OutputConfig::default());
    led.set_high();

    // initial display from state
    {
        let s = state_mutex.lock().await;
        s.update_display(&mut ac);
    }

    loop {
        ac.scan().await;

        ac.handle_input_events_to_channel(ev);

        let mut need_redraw = false;
        while redraw.try_receive().is_ok() {
            need_redraw = true;
        }

        if need_redraw {
            let s = state_mutex.lock().await;
            s.update_display(&mut ac);
        }

        Timer::after(Duration::from_millis(1)).await;
    }
}
