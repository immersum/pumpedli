pub mod lcd199;

use defmt::warn;
use display_interface_spi::SPIInterface as SpiInterface;
use embassy_rp::gpio::Output;
use embedded_graphics::Drawable;
use embedded_hal::spi::SpiDevice;
use lcd199::Lcd199;
use mipidsi::models::GC9A01;

use crate::reading::{Reading, ReadingResult, ReadingSubscriber};
use crate::scaling::ValueOutOfRange;

pub type Display<'a, T> = mipidsi::Display<SpiInterface<T, Output<'a>>, GC9A01, Output<'a>>;

pub struct Dashboard<'a, T: SpiDevice> {
    subscriber: ReadingSubscriber<'a>,
    display: Display<'a, T>,
}

impl<'a, T: SpiDevice> Dashboard<'a, T> {
    pub fn new(subscriber: ReadingSubscriber<'a>, display: Display<'a, T>) -> Self {
        Self {
            subscriber,
            display,
        }
    }

    pub async fn run(&mut self) -> ! {
        loop {
            use ReadingResult::{Err, Ok};

            let reading = self.subscriber.next_message_pure().await;
            let Reading::Moisture(control_loop, result) = reading else {
                continue;
            };

            let Some(position) = control_loop.lcd_position else {
                continue;
            };

            let lcd = match result {
                Ok(value) => Lcd199::with_value(position, value),
                Err(ValueOutOfRange::Under(_)) => Lcd199::with_value(position, i32::MIN),
                Err(ValueOutOfRange::Over(_)) => Lcd199::with_value(position, i32::MAX),
                Err(ValueOutOfRange::None) => Lcd199::new(position),
            };

            if let Result::Err(e) = lcd.draw(&mut self.display) {
                warn!("draw error: {:?}", e);
            }
        }
    }
}
