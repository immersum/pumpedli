use core::ops::RangeInclusive;

use defmt::{unwrap, warn};
use embedded_hal_async::i2c::I2c;
use fixed::types::I8F24;
use fixed_macro::types::I8F24;

use crate::dev::ads1115::{Addr, Ads1115, Channel};
use crate::reading::{Reading, ReadingPublisher, ReadingResult};
use crate::{adc, control};

pub struct Input(pub Addr, pub Channel);

pub struct Converter<'a, T: I2c> {
    ads1115: Ads1115<'a, T>,
    control_loops: [&'a control::Loop<'a>; 4],
    publisher: ReadingPublisher<'a>,
}

impl<'a, T: I2c> Converter<'a, T> {
    pub fn new(
        ads1115: Ads1115<'a, T>,
        control_loops: [&'a control::Loop<'a>; 4],
        publisher: ReadingPublisher<'a>,
    ) -> Self {
        Self {
            ads1115,
            control_loops,
            publisher,
        }
    }

    pub async fn run(mut self) -> ! {
        const SAMPLES: i32 = 5;
        const NOISE: RangeInclusive<I8F24> = I8F24!(-0.1)..=I8F24!(0.1);

        let mut average: [(I8F24, i32); 4] = Default::default();
        let mut results: [Option<ReadingResult<i32>>; 4] = Default::default();

        loop {
            let control_loops = self.control_loops.into_iter();

            let zip = control_loops
                .zip(average.iter_mut())
                .zip(results.iter_mut());

            for ((control_loop, (ref mut average, ref mut count)), result) in zip {
                let control::Loop { adc_input, .. } = *control_loop;
                let adc::Input(_, channel) = *adc_input;

                let Ok(voltage) = self.ads1115.read_voltage(channel).await else {
                    warn!("read error");
                    continue;
                };

                let delta = voltage - *average;

                *count += 1;
                *average += unwrap!(delta.checked_div_int(*count));

                let scaling = control_loop.scaling.lock().await;
                let new_result = scaling.convert_voltage(average);

                if *count < SAMPLES {
                    if *count % 2 == 0 || NOISE.contains(&delta) {
                        continue;
                    }
                } else {
                    *count = 1;
                    *average = voltage;
                }

                if result.replace(new_result).is_some_and(|r| r == new_result) {
                    continue;
                }

                let reading = Reading::Moisture(control_loop, new_result);

                self.publisher.publish(reading).await;
            }
        }
    }
}
