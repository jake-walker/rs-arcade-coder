#![no_std]
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Level, Output, OutputConfig, OutputPin},
    peripheral::Peripheral,
    spi::master::Spi,
    Blocking,
};

const CHANNEL_COUNT: usize = 6;

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
            channel_select_delay: Duration::from_micros(50),
            latch_duration: Duration::from_micros(50),
            display_buffer: [[255; 9]; 6],
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
        Timer::after(self.channel_select_delay).await;
    }

    pub async fn clear_display(&mut self) {
        self.display_buffer = [[255; 9]; 6];
    }

    pub async fn draw(&mut self) -> () {
        self.display_oe.set_low();
        self.display_latch.set_low();

        for i in 0_usize..6_usize {
            self.set_channel(Some(i)).await;

            self.display_spi
                .write(&self.display_buffer[i])
                .expect("could not write display data");
        }

        self.set_channel(None).await;
        self.display_latch.set_high();
        Timer::after(self.latch_duration).await;
        self.display_latch.set_low();
        self.display_oe.set_high();
    }
}
