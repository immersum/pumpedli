use core::fmt;
use core::ops::RangeToInclusive;

use fixed::traits::FromFixed;
use fixed::types::I8F24;
use fixed_macro::types::I8F24;

use crate::reading::ReadingResult;

pub struct Scaling {
    voltage_at_0: I8F24,
    scaling_factor: I8F24,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ValueOutOfRange {
    Under(i32),
    Over(i32),
    None,
}

impl fmt::Display for ValueOutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Under(min) => write!(f, "value below minimum of {min}: voltage is too high"),
            Self::Over(max) => write!(f, "value above maximum of {max}: voltage is too low"),
            Self::None => write!(f, "no value"),
        }
    }
}

impl Scaling {
    pub const TYPE0_5V: Self = Self::new(I8F24!(3.3), I8F24!(1.5));
    pub const TYPE0_3V3: Self = Self::new(I8F24!(2.178), I8F24!(0.99));

    pub const fn new(voltage_at_0: I8F24, voltage_at_100: I8F24) -> Self {
        Self {
            voltage_at_0,
            scaling_factor: I8F24!(100).saturating_div(voltage_at_100.saturating_sub(voltage_at_0)),
        }
    }

    pub fn convert_voltage(&self, voltage: &I8F24) -> ReadingResult<i32> {
        const NO_SENSOR: RangeToInclusive<I8F24> = ..=I8F24!(0.9);
        const LO_CUTOFF: i32 = -5;
        const HI_CUTOFF: i32 = 106;

        if NO_SENSOR.contains(voltage) {
            return ReadingResult::Err(ValueOutOfRange::None);
        }

        let value = i32::from_fixed((voltage - self.voltage_at_0).wide_mul(self.scaling_factor));

        match value {
            i32::MIN..LO_CUTOFF => ReadingResult::Err(ValueOutOfRange::Under(LO_CUTOFF)),
            LO_CUTOFF..HI_CUTOFF => ReadingResult::Ok(value.clamp(0, 100)),
            HI_CUTOFF..=i32::MAX => ReadingResult::Err(ValueOutOfRange::Over(HI_CUTOFF)),
        }
    }
}
