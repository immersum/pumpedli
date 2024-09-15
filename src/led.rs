use embassy_futures::select::select;
use embassy_futures::select::Either::First;
use embassy_rp::gpio::Output;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};

pub enum Mode {
    Off,
    OffOn(Duration, Duration),
    On,
    OnOff(Duration, Duration),
}

pub struct Blinker<'a> {
    signal: &'a Signal<CriticalSectionRawMutex, Mode>,
    led: Output<'a>,
}

impl<'a> Blinker<'a> {
    pub fn new(signal: &'a Signal<CriticalSectionRawMutex, Mode>, led: Output<'a>) -> Self {
        Self { signal, led }
    }

    pub async fn run(mut self) -> ! {
        loop {
            use Mode::{Off, OffOn, On, OnOff};

            let mode = self.signal.wait().await;

            match mode {
                Off | On => {
                    self.led.set_level(matches!(mode, On).into());
                    continue;
                }
                OffOn(a, b) | OnOff(a, b) => loop {
                    self.led.set_level(matches!(mode, OnOff(_, _)).into());
                    if let First(mode) = select(self.signal.wait(), Timer::after(a)).await {
                        self.signal.signal(mode);
                        break;
                    }

                    self.led.set_level(matches!(mode, OffOn(_, _)).into());
                    if let First(mode) = select(self.signal.wait(), Timer::after(b)).await {
                        self.signal.signal(mode);
                        break;
                    }
                },
            }
        }
    }
}
