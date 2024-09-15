use core::cmp;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};

use crate::control;
use crate::scaling::ValueOutOfRange;

#[derive(Clone, Copy)]
pub enum ReadingResult<T> {
    Ok(T),
    Err(ValueOutOfRange),
}

#[derive(Clone)]
pub enum Reading<'a> {
    Moisture(&'a control::Loop<'a>, ReadingResult<i32>),
    Temperature(f32),
}

pub type ReadingPublisher<'a> = Publisher<'a, CriticalSectionRawMutex, Reading<'a>, 1, 2, 4>;
pub type ReadingSubscriber<'a> = Subscriber<'a, CriticalSectionRawMutex, Reading<'a>, 1, 2, 4>;
pub type ReadingPubSubChannel<'a> = PubSubChannel<CriticalSectionRawMutex, Reading<'a>, 1, 2, 4>;

impl<T: PartialOrd> PartialEq for ReadingResult<T> {
    fn eq(&self, other: &Self) -> bool {
        use ReadingResult::{Err, Ok};

        match (self, other) {
            (Ok(value), Ok(other)) => value.eq(other),
            (Err(ValueOutOfRange::Under(min1)), Err(ValueOutOfRange::Under(min2))) => min1 == min2,
            (Err(ValueOutOfRange::Over(max1)), Err(ValueOutOfRange::Over(max2))) => max1 == max2,
            (Err(ValueOutOfRange::None), Err(ValueOutOfRange::None)) => true,
            _ => false,
        }
    }
}

impl<T: PartialOrd> PartialOrd for ReadingResult<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        use cmp::Ordering::{Equal, Greater, Less};
        use ReadingResult::{Err, Ok};

        match self {
            Ok(value) => match other {
                Ok(other) => value.partial_cmp(other),
                Err(ValueOutOfRange::Under(_)) => Some(Greater),
                Err(ValueOutOfRange::Over(_)) => Some(Less),
                Err(ValueOutOfRange::None) => None,
            },
            Err(ValueOutOfRange::Under(min1)) => match other {
                Ok(_) => Some(Less),
                Err(ValueOutOfRange::Under(min2)) if min1 == min2 => Some(Equal),
                Err(ValueOutOfRange::Under(_)) => None,
                Err(ValueOutOfRange::Over(_)) => Some(Less),
                Err(ValueOutOfRange::None) => None,
            },
            Err(ValueOutOfRange::Over(max1)) => match other {
                Ok(_) => Some(Greater),
                Err(ValueOutOfRange::Under(_)) => Some(Greater),
                Err(ValueOutOfRange::Over(max2)) if max1 == max2 => Some(Equal),
                Err(ValueOutOfRange::Over(_)) => None,
                Err(ValueOutOfRange::None) => None,
            },
            Err(ValueOutOfRange::None) => None,
        }
    }
}
