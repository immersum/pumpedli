use core::future;

use defmt::{trace, warn, Format};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant};
use palette::{named, GetHue, Srgb};

use crate::control::{Action, ActionPublisher};
use crate::reading::{Reading, ReadingResult, ReadingSubscriber};
use crate::scaling::ValueOutOfRange;
use crate::{adc, led, rgb};

#[derive(Default)]
pub struct Program(pub ProgramConfig, pub ProgramState);

pub struct ProgramConfig {
    pub low_threshold: i32,
    pub high_threshold: i32,
    pub run_duration: Duration,
    pub pause_duration: Duration,
}

#[derive(Default)]
#[non_exhaustive]
pub enum ProgramState {
    #[default]
    Stopped,
    DoingRuns {
        result: ReadingResult<i32>,
    },
    Faulted {
        fault: ProgramFault,
    },
}

#[non_exhaustive]
pub enum ProgramFault {
    WaterNotRunning,
}

pub struct Regulator<'a> {
    subscriber: ReadingSubscriber<'a>,
    publisher: ActionPublisher<'a>,
    led: &'a Signal<CriticalSectionRawMutex, led::Mode>,
    rgb: &'a Signal<CriticalSectionRawMutex, rgb::Mode>,
}

impl Default for ProgramConfig {
    fn default() -> Self {
        Self {
            low_threshold: 60,
            high_threshold: 90,
            run_duration: Duration::from_secs(3),
            pause_duration: Duration::from_secs(120),
        }
    }
}

impl Format for ProgramFault {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Self::WaterNotRunning => defmt::write!(fmt, "water is not running"),
        }
    }
}

impl<'a> Regulator<'a> {
    pub fn new(
        subscriber: ReadingSubscriber<'a>,
        publisher: ActionPublisher<'a>,
        led: &'a Signal<CriticalSectionRawMutex, led::Mode>,
        rgb: &'a Signal<CriticalSectionRawMutex, rgb::Mode>,
    ) -> Self {
        Self {
            subscriber,
            publisher,
            led,
            rgb,
        }
    }

    pub async fn run(&mut self) -> ! {
        loop {
            use led::Mode::Off;
            use rgb::Mode::Color;

            let reading = self.subscriber.next_message_pure().await;
            let Reading::Moisture(control_loop, result) = reading else {
                continue;
            };

            let t_ms = Instant::now().as_millis();
            let adc::Input(addr, channel) = *control_loop.adc_input;

            match result {
                ReadingResult::Ok(value) => {
                    log::info!("{t_ms} ms; addr {addr}; channel {channel}; value {value}");
                    trace!("addr {}; channel {}; value {}", addr, channel, value);
                }
                ReadingResult::Err(e) => {
                    log::info!("{t_ms} ms; addr {addr}; channel {channel}; {e}");
                    trace!("addr {}; channel {}; no value", addr, channel);
                }
            }

            trace!("waiting to lock program...");
            let mut program = control_loop.program.lock().await;
            let Some(Program(ref config, ref mut state)) = *program else {
                trace!("no program is available");
                continue;
            };

            match state {
                ProgramState::Stopped => {
                    let needs_water = match result {
                        ReadingResult::Ok(value) => value < config.low_threshold,
                        ReadingResult::Err(ValueOutOfRange::Under(_)) => true,
                        ReadingResult::Err(ValueOutOfRange::Over(_)) => false,
                        ReadingResult::Err(ValueOutOfRange::None) => false,
                    };

                    if needs_water {
                        self.publisher.publish(Action::RunWater(reading)).await;
                        self.led.signal(Off);
                    }
                }
                ProgramState::DoingRuns { .. } => {
                    let needs_water = match result {
                        ReadingResult::Ok(value) => value < config.high_threshold,
                        ReadingResult::Err(ValueOutOfRange::Under(_)) => true,
                        ReadingResult::Err(ValueOutOfRange::Over(_)) => false,
                        ReadingResult::Err(ValueOutOfRange::None) => false,
                    };

                    if !needs_water {
                        self.publisher.publish(Action::Stop).await;
                        self.led.signal(Off);
                    }

                    *state = ProgramState::DoingRuns { result };
                }
                ProgramState::Faulted { fault } => {
                    warn!("program is halted due to a fault: {}", fault);
                    self.rgb.signal(Color {
                        hue: Srgb::<f32>::from(named::RED).get_hue(),
                        value: 0.1,
                    });

                    future::pending().await
                }
            }
        }
    }
}
