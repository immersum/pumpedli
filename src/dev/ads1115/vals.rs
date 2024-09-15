use core::fmt;
use defmt::Format;

pub struct Reg();

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Addr {
    Gnd = 0b1001000,
    Vdd = Addr::Gnd as u8 + 1,
    Sda = Addr::Gnd as u8 + 2,
    Scl = Addr::Gnd as u8 + 3,
}

#[derive(Clone, Copy, Format)]
pub enum Channel {
    A0,
    A1,
    A2,
    A3,
}

impl Reg {
    pub const RESULT: [u8; 1] = [0];
    pub const CONFIG: [u8; 1] = [1];
    pub const LO_THRESH: [u8; 1] = [2];
    pub const HI_THRESH: [u8; 1] = [3];
}

impl Format for Addr {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Self::Gnd => defmt::write!(fmt, "GND"),
            Self::Vdd => defmt::write!(fmt, "VDD"),
            Self::Sda => defmt::write!(fmt, "SDA"),
            Self::Scl => defmt::write!(fmt, "SCL"),
        }
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gnd => write!(f, "GND"),
            Self::Vdd => write!(f, "VDD"),
            Self::Sda => write!(f, "SDA"),
            Self::Scl => write!(f, "SCL"),
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A0 => write!(f, "A0"),
            Self::A1 => write!(f, "A1"),
            Self::A2 => write!(f, "A2"),
            Self::A3 => write!(f, "A3"),
        }
    }
}
