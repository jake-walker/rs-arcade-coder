#![no_std]
#![no_main]

use core::{
    cell::RefCell,
    sync::atomic::{AtomicU32, Ordering},
};

use arcadecoder_hw::{
    font::{FONT_5X5, FONT_5X5_SIZE},
    ArcadeCoder, BLUE, GREEN,
};
use embassy_executor::Spawner;
use embassy_futures::{
    select::{select, Either},
    yield_now,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Level, Output},
    spi::master::Spi,
    time::RateExtU32,
    timer::timg::TimerGroup,
};
use esp_println::println;

static SCORE_A: AtomicU32 = AtomicU32::new(0);
static SCORE_B: AtomicU32 = AtomicU32::new(0);

static BUTTON_PRESSED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

static AC_MUTEX: Mutex<CriticalSectionRawMutex, RefCell<Option<ArcadeCoder>>> =
    Mutex::new(RefCell::new(None));

fn update_display(ac: &mut ArcadeCoder<'_>, score_a: u32, score_b: u32) {
    ac.clear_display();

    ac.draw_digit(score_a % 10, FONT_5X5, FONT_5X5_SIZE, (0, 6), GREEN);
    ac.draw_digit((score_a / 10) % 10, FONT_5X5, FONT_5X5_SIZE, (0, 0), GREEN);
    ac.draw_digit(score_b % 10, FONT_5X5, FONT_5X5_SIZE, (7, 6), BLUE);
    ac.draw_digit((score_b / 10) % 10, FONT_5X5, FONT_5X5_SIZE, (7, 0), BLUE);
}

#[embassy_executor::task]
async fn display_task() {
    Timer::after(Duration::from_secs(1)).await;

    loop {
        // wait for the button to be pressed or 100 microseconds
        match select(BUTTON_PRESSED.wait(), Timer::after_micros(100)).await {
            // if the button was pressed, update the display and draw
            Either::First(_) => {
                let score_a_value = SCORE_A.load(Ordering::Relaxed);
                let score_b_value = SCORE_B.load(Ordering::Relaxed);

                let ac_guard = AC_MUTEX.lock().await;
                let mut ac_ref = ac_guard.borrow_mut();

                if let Some(ac) = ac_ref.as_mut() {
                    update_display(ac, score_a_value, score_b_value);
                    ac.draw().await;
                }

                yield_now().await;
            }
            // ...otherwise just draw
            Either::Second(_) => {
                let ac_guard = AC_MUTEX.lock().await;
                let mut ac_ref = ac_guard.borrow_mut();

                if let Some(ac) = ac_ref.as_mut() {
                    ac.draw().await;
                }

                yield_now().await;
            }
        }
    }
}

#[embassy_executor::task]
async fn button_task() {
    loop {
        {
            let ac_guard = AC_MUTEX.lock().await;

            // if any rows are pressed
            if ac_guard
                .borrow()
                .as_ref()
                .unwrap()
                .read_row_values()
                .iter()
                .any(|v| *v)
            {
                // borrow the hardware
                let mut ac_ref = ac_guard.borrow_mut();

                if let Some(ac) = ac_ref.as_mut() {
                    // get the currently pressed coordinates
                    if let Some(coords) = ac.read_buttons().await {
                        if coords.1 == 11 {
                            // if the bottom row, reset the scores
                            SCORE_A.store(0, Ordering::Relaxed);
                            SCORE_B.store(0, Ordering::Relaxed);
                        } else if coords.0 < 6 && coords.1 < 6 {
                            // if top-left, add 1 to score a
                            SCORE_A.fetch_add(1, Ordering::Relaxed);
                        } else if coords.0 < 6 {
                            // if bottom-left, subtract 1 from score a
                            SCORE_A.fetch_sub(1, Ordering::Relaxed);
                        } else if coords.1 < 6 {
                            // if top-right, add 1 to score b
                            SCORE_B.fetch_add(1, Ordering::Relaxed);
                        } else {
                            // if bottom-right, subtract 1 from score b
                            SCORE_B.fetch_sub(1, Ordering::Relaxed);
                        }

                        // signal a button was pressed
                        BUTTON_PRESSED.signal(());

                        // debounce
                        Timer::after_millis(200).await;
                    }
                }
            }
        }

        // time for the display to do things
        Timer::after_millis(10).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    println!("Init!");
    let p = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let mut ac = ArcadeCoder::new(
        Spi::new(
            p.SPI2,
            esp_hal::spi::master::Config::default()
                .with_frequency(200_u32.kHz())
                .with_mode(esp_hal::spi::Mode::_0)
                .with_write_bit_order(esp_hal::spi::BitOrder::MsbFirst),
        )
        .expect("could not create spi")
        .with_mosi(p.GPIO5)
        .with_sck(p.GPIO17),
        p.GPIO19,
        p.GPIO18,
        p.GPIO21,
        p.GPIO4,
        p.GPIO16,
        p.GPIO39,
        p.GPIO36,
        p.GPIO35,
        p.GPIO34,
        p.GPIO33,
        p.GPIO32,
    );

    update_display(&mut ac, 0, 0);

    *AC_MUTEX.lock().await.borrow_mut() = Some(ac);

    let mut led = Output::new(p.GPIO22, Level::Low);
    led.set_high();

    spawner
        .spawn(display_task())
        .expect("could not spawn display task");
    spawner
        .spawn(button_task())
        .expect("could not spawn button task");
}
