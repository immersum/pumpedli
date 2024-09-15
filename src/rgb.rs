use embassy_futures::select::select;
use embassy_futures::select::Either::First;
use embassy_rp::pio::Instance;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use palette::convert::FromColorUnclamped;
use palette::{DarkenAssign, Hsv, LightenAssign, RgbHue, ShiftHueAssign, Srgb};

use crate::dev::ws2812::Ws2812;

pub enum Mode {
    Off,
    White {
        value: f32,
    },
    Color {
        hue: RgbHue<f32>,
        value: f32,
    },
    Rainbow {
        period: Duration,
        value: f32,
    },
    Breathe {
        period: Duration,
        hue: RgbHue<f32>,
        value: f32,
    },
}

pub struct Control<'d, P: Instance, const S: usize> {
    signal: &'d Signal<CriticalSectionRawMutex, Mode>,
    ws2812: Ws2812<'d, P, S>,
}

impl<'d, P: Instance, const S: usize> Control<'d, P, S> {
    pub fn new(
        signal: &'d Signal<CriticalSectionRawMutex, Mode>,
        ws2812: Ws2812<'d, P, S>,
    ) -> Self {
        Self { signal, ws2812 }
    }

    pub async fn run(mut self) -> ! {
        const CYCLE_DIVS: u16 = 360;

        loop {
            use Mode::{Breathe, Color, Off, Rainbow, White};

            let mode = self.signal.wait().await;

            let mut color = match mode {
                Off => Hsv::new(0.0, 0.0, 0.0),
                White { value } => Hsv::new(0.0, 0.0, value),
                Color { hue, value } => Hsv::new(hue, 1.0, value),
                Rainbow { value, .. } => Hsv::new(0.0, 1.0, value),
                Breathe { hue, .. } => Hsv::new(hue, 1.0, 0.0),
            };

            let mut colors = [Srgb::from_color_unclamped(color).into()];

            match mode {
                Off | White { .. } | Color { .. } => {
                    self.ws2812.write_mut(&mut colors).await;
                    continue;
                }
                Rainbow { period, .. } => {
                    let delta_t = period / u32::from(CYCLE_DIVS);
                    let delta_hue = 360.0 / f32::from(CYCLE_DIVS);

                    loop {
                        let future = async {
                            self.ws2812.write_mut(&mut colors).await;
                            self.signal.wait().await
                        };

                        if let First(mode) = select(future, Timer::after(delta_t)).await {
                            self.signal.signal(mode);
                            break;
                        }

                        color.shift_hue_assign(delta_hue);
                        colors[0] = Srgb::from_color_unclamped(color).into();
                    }
                }
                Breathe { period, value, .. } => {
                    let delta_t = period / u32::from(CYCLE_DIVS);
                    let delta_value_a = value / f32::from(CYCLE_DIVS / 2);
                    let delta_value_b = value / f32::from(CYCLE_DIVS / 2 + CYCLE_DIVS % 2);

                    'breathing: loop {
                        for _ in 0..(CYCLE_DIVS / 2) {
                            let future = async {
                                self.ws2812.write_mut(&mut colors).await;
                                self.signal.wait().await
                            };

                            if let First(mode) = select(future, Timer::after(delta_t)).await {
                                self.signal.signal(mode);
                                break 'breathing;
                            }

                            color.lighten_fixed_assign(delta_value_a);
                            colors[0] = Srgb::from_color_unclamped(color).into();
                        }

                        for _ in 0..(CYCLE_DIVS / 2 + CYCLE_DIVS % 2) {
                            let future = async {
                                self.ws2812.write_mut(&mut colors).await;
                                self.signal.wait().await
                            };

                            if let First(mode) = select(future, Timer::after(delta_t)).await {
                                self.signal.signal(mode);
                                break 'breathing;
                            }

                            color.darken_fixed_assign(delta_value_b);
                            colors[0] = Srgb::from_color_unclamped(color).into();
                        }
                    }
                }
            }
        }
    }
}
