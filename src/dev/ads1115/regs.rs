use bilge::prelude::*;
use fixed::types::I8F24;
use fixed_macro::types::I8F24;

#[bitsize(16)]
#[derive(FromBits, DebugBits, Clone)]
pub struct Config {
    pub comparator: CompareConfig,
    pub data_rate: DataRate,
    pub op_mode: OperateMode,
    pub op_amp_gain: OpAmpGain,
    pub input_mux: InputMux,
    pub op_status: bool,
}

#[bitsize(3)]
#[derive(FromBits, Debug)]
pub enum InputMux {
    A0A1,
    A0A3,
    A1A3,
    A2A3,
    A0Gnd,
    A1Gnd,
    A2Gnd,
    A3Gnd,
}

#[bitsize(3)]
#[derive(FromBits, Debug)]
pub enum OpAmpGain {
    Upto6V144,
    Upto4V096,
    Upto2V048,
    Upto1V024,
    Upto0V512,
    #[fallback]
    Upto0V256,
}

#[bitsize(1)]
#[derive(FromBits, Debug)]
pub enum OperateMode {
    Continuous,
    SingleShot,
}

#[bitsize(3)]
#[derive(FromBits, Debug)]
pub enum DataRate {
    Sps8,
    Sps16,
    Sps32,
    Sps64,
    Sps128,
    Sps250,
    Sps475,
    Sps860,
}

#[bitsize(5)]
#[derive(FromBits, DebugBits)]
pub struct CompareConfig {
    pub comp_queue: CompareQueue,
    pub output_lat: bool,
    pub output_pol: Polarity,
    pub comp_mode: CompareMode,
}

#[bitsize(1)]
#[derive(FromBits, Debug)]
pub enum CompareMode {
    AboveThreshold,
    OutsideWindow,
}

#[bitsize(1)]
#[derive(FromBits, Debug)]
pub enum Polarity {
    ActiveLow,
    ActiveHigh,
}

#[bitsize(2)]
#[derive(FromBits, Debug)]
pub enum CompareQueue {
    CompareOne,
    CompareTwo,
    CompareFour,
    DisableCompare,
}

impl From<OpAmpGain> for I8F24 {
    fn from(value: OpAmpGain) -> Self {
        match value {
            OpAmpGain::Upto6V144 => I8F24!(6.144),
            OpAmpGain::Upto4V096 => I8F24!(4.096),
            OpAmpGain::Upto2V048 => I8F24!(2.048),
            OpAmpGain::Upto1V024 => I8F24!(1.024),
            OpAmpGain::Upto0V512 => I8F24!(0.512),
            OpAmpGain::Upto0V256 => I8F24!(0.256),
        }
    }
}

impl From<DataRate> for u64 {
    fn from(value: DataRate) -> Self {
        match value {
            DataRate::Sps8 => 8,
            DataRate::Sps16 => 16,
            DataRate::Sps32 => 32,
            DataRate::Sps64 => 64,
            DataRate::Sps128 => 128,
            DataRate::Sps250 => 250,
            DataRate::Sps475 => 475,
            DataRate::Sps860 => 860,
        }
    }
}
