#![no_std]
#![no_main]

use arcadecoder_hw::ArcadeCoder;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    spi::master::Spi,
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::println;

#[embassy_executor::task]
async fn display(mut ac: ArcadeCoder<'static>) {
    Timer::after(Duration::from_secs(1)).await;

    loop {
        ac.draw().await;
        Timer::after(Duration::from_millis(1)).await;
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
                .with_frequency(Rate::from_khz(200))
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
    );

    let mut led = Output::new(p.GPIO22, Level::Low, OutputConfig::default());
    led.set_high();

    spawner.spawn(display(ac)).expect("could not spawn task");
}
