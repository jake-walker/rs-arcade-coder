#![no_std]
#![no_main]

use arcadecoder_hw::{
    font::{FONT_5X5, FONT_5X5_SIZE},
    ArcadeCoder, WHITE,
};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Level, Output},
    spi::master::Spi,
    time::RateExtU32,
    timer::timg::TimerGroup,
};
use esp_println::println;

#[embassy_executor::task]
async fn display(mut ac: ArcadeCoder<'static>) {
    Timer::after(Duration::from_secs(1)).await;

    loop {
        for n in 0_u32..99_u32 {
            ac.clear_display();
            let digit1 = n % 10;
            let digit2 = (n / 10) % 10;
            ac.draw_digit(digit1, FONT_5X5, FONT_5X5_SIZE, (6, 0), WHITE);
            ac.draw_digit(digit2, FONT_5X5, FONT_5X5_SIZE, (0, 0), WHITE);
            for _ in 0..50 {
                ac.draw().await;
                Timer::after_millis(1).await;
            }
            if let Some(coords) = ac.read_buttons().await {
                println!("Button pressed at {:?}", coords);
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

    let ac = ArcadeCoder::new(
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

    let mut led = Output::new(p.GPIO22, Level::Low);
    led.set_high();

    spawner.spawn(display(ac)).expect("could not spawn task");
}
