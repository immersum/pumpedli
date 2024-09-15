use pumpedli::adc;
use pumpedli::dev::ads1115::{Addr, Channel};

pub const INPUTS: [adc::Input; 16] = [
    adc::Input(Addr::Gnd, Channel::A0),
    adc::Input(Addr::Gnd, Channel::A1),
    adc::Input(Addr::Gnd, Channel::A2),
    adc::Input(Addr::Gnd, Channel::A3),
    adc::Input(Addr::Vdd, Channel::A0),
    adc::Input(Addr::Vdd, Channel::A1),
    adc::Input(Addr::Vdd, Channel::A2),
    adc::Input(Addr::Vdd, Channel::A3),
    adc::Input(Addr::Sda, Channel::A0),
    adc::Input(Addr::Sda, Channel::A1),
    adc::Input(Addr::Sda, Channel::A2),
    adc::Input(Addr::Sda, Channel::A3),
    adc::Input(Addr::Scl, Channel::A0),
    adc::Input(Addr::Scl, Channel::A1),
    adc::Input(Addr::Scl, Channel::A2),
    adc::Input(Addr::Scl, Channel::A3),
];
