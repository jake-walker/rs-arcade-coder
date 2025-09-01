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
use static_cell::StaticCell;

const A_COLOR: Color = GREEN;
const B_COLOR: Color = MAGENTA;

// mutexes and channels for sharing data between task and main thread
static EVENT_CH: StaticCell<Channel<NoopRawMutex, ButtonEvent, 64>> = StaticCell::new();
static REDRAW_CH: StaticCell<Channel<NoopRawMutex, (), 4>> = StaticCell::new();
static STATE_MUTEX: StaticCell<Mutex<NoopRawMutex, State>> = StaticCell::new();

// add the required esp app descriptor
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
}

// lightweight snapshot type and helpers to avoid holding the mutex while rendering.
type Snapshot = (u8, u8, u8, bool, bool); // (score_a, score_b, win_threshold, a_winner, b_winner)

fn snapshot_from_state(s: &State) -> Snapshot {
    (
        s.score_a,
        s.score_b,
        s.win_threshold,
        s.a_winner,
        s.b_winner,
    )
}

// render a snapshot to the display
fn render_snapshot(ac: &mut ArcadeCoder<'_>, snap: Snapshot) {
    let (score_a, score_b, win_threshold, a_winner, b_winner) = snap;

    ac.clear();

    let mut a_text_color = A_COLOR;
    let mut b_text_color = B_COLOR;

    if a_winner {
        ac.draw_rect((0, 0), (11, 11), A_COLOR);
        a_text_color = WHITE;
    } else if b_winner {
        ac.draw_rect((0, 0), (11, 11), B_COLOR);
        b_text_color = WHITE;
    }

    ac.draw_digit(
        (score_a % 10).into(),
        FONT_5X5,
        FONT_5X5_SIZE,
        (0, 6),
        a_text_color,
    );
    ac.draw_digit(
        ((score_a / 10) % 10).into(),
        FONT_5X5,
        FONT_5X5_SIZE,
        (0, 0),
        a_text_color,
    );
    ac.draw_digit(
        (score_b % 10).into(),
        FONT_5X5,
        FONT_5X5_SIZE,
        (7, 6),
        b_text_color,
    );
    ac.draw_digit(
        ((score_b / 10) % 10).into(),
        FONT_5X5,
        FONT_5X5_SIZE,
        (7, 0),
        b_text_color,
    );

    if win_threshold == 21 {
        ac.set_pixel((5, 11), RED);
        ac.set_pixel((6, 11), RED);
    }
}

// task for updating the state based on button press events
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
            let mut mutated = false;

            match event {
                ButtonEvent::Pressed(x, y) => {
                    if y == 11 && (x == 5 || x == 6) {
                        if s.win_threshold == 11 {
                            s.win_threshold = 21
                        } else {
                            s.win_threshold = 11
                        }
                        s.check_win();
                        mutated = true;
                    } else if y == 11 {
                        // if the bottom row, reset the scores
                        s.reset();
                        mutated = true;
                    } else if x < 6 && y < 6 {
                        // if top-left, add 1 to score a
                        s.inc_score_a();
                        mutated = true;
                    } else if x < 6 {
                        // if bottom-left, subtract 1 from score a
                        s.dec_score_a();
                        mutated = true;
                    } else if y < 6 {
                        // if top-right, add 1 to score b
                        s.inc_score_b();
                        mutated = true;
                    } else {
                        // if bottom-right, subtract 1 from score b
                        s.dec_score_b();
                        mutated = true;
                    }
                }
                ButtonEvent::Released(_, _) => {}
            }

            // if the state was mutated, trigger a redraw on the main thread
            if mutated {
                let _ = redraw.try_send(());
            }
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    println!("Init!");
    // initialize the esp
    let p = esp_hal::init(esp_hal::Config::default());

    // setup embassy with a hardware timer
    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // setup the arcade coder instance, passing in the required pins
    let mut ac = ArcadeCoder::new(
        p.SPI2, p.GPIO19, p.GPIO18, p.GPIO21, p.GPIO4, p.GPIO16, p.GPIO5, p.GPIO17, p.GPIO39,
        p.GPIO36, p.GPIO35, p.GPIO34, p.GPIO33, p.GPIO32,
    );

    // initialize mutexes and channels
    let ev: &'static Channel<NoopRawMutex, ButtonEvent, 64> = EVENT_CH.init(Channel::new());
    let redraw: &'static Channel<NoopRawMutex, (), 4> = REDRAW_CH.init(Channel::new());
    let state_mutex: &'static Mutex<NoopRawMutex, State> =
        STATE_MUTEX.init(Mutex::new(State::new()));

    // start a task for handling state updates
    spawner.spawn(state_task(ev, redraw, state_mutex)).unwrap();

    // turn on the led
    let mut led = Output::new(p.GPIO22, Level::Low, OutputConfig::default());
    led.set_high();

    // initial display from state (take a snapshot and render outside the lock)
    {
        let s = state_mutex.lock().await;
        let snap = snapshot_from_state(&s);
        drop(s);
        render_snapshot(&mut ac, snap);
    }

    // main loop
    loop {
        // draw the display and get button press inputs
        ac.scan();

        // handle debounced button events from the library, passing through to the input channel
        ac.handle_input_events_to_channel(ev);

        // check if a redraw is required - if the state has changed
        let mut need_redraw = false;
        while redraw.try_receive().is_ok() {
            need_redraw = true;
        }

        // update the display if a redraw is required
        if need_redraw {
            // take a snapshot while holding the mutex, then render outside
            let s = state_mutex.lock().await;
            let snap = snapshot_from_state(&s);
            drop(s);
            render_snapshot(&mut ac, snap);
        }

        Timer::after(Duration::from_millis(1)).await;
    }
}
