use core::cmp;

use defmt::{debug, trace, warn};
use embassy_rp::gpio::Output;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::mutex::Mutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};
use embassy_sync::signal::Signal;
use embassy_time::{with_timeout, Duration, TimeoutError};
use palette::{named, GetHue, Srgb};

use crate::dev::cd4067::Cd4067;
use crate::display::lcd199::Position;
use crate::program::{Program, ProgramConfig, ProgramFault, ProgramState};
use crate::reading::Reading;
use crate::scaling::Scaling;
use crate::{adc, led, mux, rgb};

#[derive(Clone)]
#[non_exhaustive]
pub enum Action<'a> {
    RunWater(Reading<'a>),
    Stop,
}

pub type ActionPublisher<'a> = Publisher<'a, CriticalSectionRawMutex, Action<'a>, 16, 16, 1>;
pub type ActionSubscriber<'a> = Subscriber<'a, CriticalSectionRawMutex, Action<'a>, 16, 16, 1>;
pub type ActionPubSubChannel<'a> = PubSubChannel<CriticalSectionRawMutex, Action<'a>, 16, 16, 1>;

pub struct Irrigator<'a> {
    subscriber: ActionSubscriber<'a>,
    mux_output: &'a mux::Output,
    control_mutex: &'a Mutex<NoopRawMutex, (Output<'a>, Cd4067<'a>)>,
    led: &'a Signal<CriticalSectionRawMutex, led::Mode>,
    rgb: &'a Signal<CriticalSectionRawMutex, rgb::Mode>,
}

pub struct Loop<'a> {
    pub adc_input: &'a adc::Input,
    pub mux_output: &'a mux::Output,
    pub lcd_position: Option<Position>,
    pub scaling: Mutex<CriticalSectionRawMutex, Scaling>,
    pub program: Mutex<CriticalSectionRawMutex, Option<Program>>,
}

impl<'a> Irrigator<'a> {
    pub fn new(
        subscriber: ActionSubscriber<'a>,
        mux_output: &'a mux::Output,
        control_mutex: &'a Mutex<NoopRawMutex, (Output<'a>, Cd4067<'a>)>,
        led: &'a Signal<CriticalSectionRawMutex, led::Mode>,
        rgb: &'a Signal<CriticalSectionRawMutex, rgb::Mode>,
    ) -> Self {
        Self {
            subscriber,
            mux_output,
            control_mutex,
            led,
            rgb,
        }
    }

    pub async fn run(&mut self) -> ! {
        'running: loop {
            use led::Mode::OnOff;

            let action = self.subscriber.next_message_pure().await;
            let Action::RunWater(reading) = action else {
                continue;
            };

            let Reading::Moisture(control_loop, result) = reading else {
                continue;
            };

            if self.mux_output != control_loop.mux_output {
                continue;
            }

            let new_state = ProgramState::DoingRuns { result };
            let Ok(_) = control_loop.map_state_mut(|state| *state = new_state).await else {
                continue;
            };

            let closure = |other| result.partial_cmp(&other);

            loop {
                let Ok(duration) = control_loop.map_config(|c| c.run_duration).await else {
                    break;
                };

                let Err(TimeoutError) = self.run_water(duration).await else {
                    break;
                };

                let Ok(duration) = control_loop.map_config(|c| c.pause_duration).await else {
                    break;
                };

                let Err(TimeoutError) = self.pause(duration).await else {
                    break;
                };

                let future = control_loop.map_state(|state| {
                    if let ProgramState::DoingRuns { result } = *state {
                        Ok(result)
                    } else {
                        warn!("program state is not as expected");
                        Err(())
                    }
                });

                let Ok(Some(ordering)) = future.await.and_then(|result| result.map(closure)) else {
                    break;
                };

                if ordering == cmp::Ordering::Less {
                    continue;
                }

                let new_state = ProgramState::Faulted {
                    fault: ProgramFault::WaterNotRunning,
                };

                let Ok(_) = control_loop.map_state_mut(|state| *state = new_state).await else {
                    break;
                };

                continue 'running;
            }

            let new_state = ProgramState::Stopped;
            let Ok(_) = control_loop.map_state_mut(|state| *state = new_state).await else {
                continue;
            };

            self.led.signal(OnOff(
                Duration::from_millis(600),
                Duration::from_millis(2400),
            ));
        }
    }

    async fn run_water(&mut self, duration: Duration) -> Result<(), TimeoutError> {
        use led::Mode::{Off, On};
        use rgb::Mode::{Color, Off as Black};

        let mux::Output(channel) = *self.mux_output;

        trace!("waiting to gain control over channel {}...", channel);
        let mut control_mutex = self.control_mutex.lock().await;
        let (ref mut motor, ref mut cd4067) = *control_mutex;

        debug!("running water on channel {}...", channel);
        motor.set_high();
        cd4067.enable(channel);
        self.led.signal(On);
        self.rgb.signal(Color {
            hue: Srgb::<f32>::from(named::BLUE).get_hue(),
            value: 0.1,
        });

        debug!("waiting for stop signal...");
        let result = with_timeout(duration, self.wait_for_stop()).await;

        debug!("water is no longer running");
        motor.set_low();
        cd4067.disable();
        self.led.signal(Off);
        self.rgb.signal(Black);

        result
    }

    async fn pause(&mut self, duration: Duration) -> Result<(), TimeoutError> {
        use led::Mode::{Off, OnOff};

        trace!("control flow is now paused");
        self.led.signal(OnOff(
            Duration::from_millis(150),
            Duration::from_millis(350),
        ));

        trace!("waiting for stop signal...");
        let result = with_timeout(duration, self.wait_for_stop()).await;

        trace!("control flow is no longer paused");
        self.led.signal(Off);

        result
    }

    async fn wait_for_stop(&mut self) {
        loop {
            let action = self.subscriber.next_message_pure().await;
            if let Action::Stop = action {
                trace!("received command to stop running water");
                return;
            };
        }
    }
}

impl Loop<'_> {
    async fn map_config<T>(&self, f: impl FnOnce(&ProgramConfig) -> T) -> Result<T, ()> {
        trace!("control loop is waiting to lock program and map config...");
        let program = self.program.lock().await;
        let Some(Program(ref config, _)) = *program else {
            warn!("program is no longer available");
            return Err(());
        };

        Ok(f(config))
    }

    async fn map_state<T>(&self, f: impl FnOnce(&ProgramState) -> T) -> Result<T, ()> {
        trace!("control loop is waiting to lock program and map state...");
        let program = self.program.lock().await;
        let Some(Program(_, ref state)) = *program else {
            warn!("program is no longer available");
            return Err(());
        };

        Ok(f(state))
    }

    async fn map_state_mut<T>(&self, f: impl FnOnce(&mut ProgramState) -> T) -> Result<T, ()> {
        trace!("control loop is waiting to lock program and map state...");
        let mut program = self.program.lock().await;
        let Some(Program(_, ref mut state)) = *program else {
            warn!("program is no longer available");
            return Err(());
        };

        Ok(f(state))
    }
}
