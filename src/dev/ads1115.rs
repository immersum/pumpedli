mod regs;
mod vals;

use defmt::{trace, warn};
use embassy_rp::gpio::Input;
use embassy_time::{with_timeout, Duration, Timer};
use embedded_hal_async::i2c::I2c;
use fixed::types::I8F24;
use regs::*;
use vals::*;

pub use vals::{Addr, Channel};

pub struct Ads1115<'a, T: I2c> {
    i2c: T,
    addr: Addr,
    config: Config,
    rdy: Input<'a>,
}

impl<'a, T: I2c> Ads1115<'a, T> {
    pub async fn new(mut i2c: T, addr: Addr, rdy: Input<'a>) -> Result<Self, T::Error> {
        Timer::after_micros(55).await;
        let mut config = Self::read_config(&mut i2c, addr).await?;
        config.set_op_amp_gain(OpAmpGain::Upto4V096);
        config.set_data_rate(DataRate::Sps64);

        let mut comparator = config.comparator();
        comparator.set_comp_queue(CompareQueue::CompareOne);
        config.set_comparator(comparator);
        Self::write_thresh(&mut i2c, addr, (0, -1)).await?;

        Ok(Self {
            i2c,
            addr,
            config,
            rdy,
        })
    }

    pub async fn read_voltage(&mut self, channel: Channel) -> Result<I8F24, T::Error> {
        let gain = self.config.op_amp_gain();
        let unit = I8F24::from(gain) >> 15;
        let result = self.read_channel_raw(channel).await?;
        Ok(i32::from(result) * unit)
    }

    pub async fn read_channel_raw(&mut self, channel: Channel) -> Result<i16, T::Error> {
        let mut config = self.config.clone();
        let hz = config.data_rate().into();
        config.set_input_mux(channel.into());
        Self::write_config(&mut self.i2c, self.addr, config).await?;

        trace!("waiting for low on ALERT/RDY pin...");

        let future = self.rdy.wait_for_low();
        if with_timeout(Duration::from_hz(hz), future).await.is_err() {
            warn!("ADC lagged");
        }

        Self::read_result(&mut self.i2c, self.addr).await
    }

    async fn read_result(i2c: &mut T, addr: Addr) -> Result<i16, T::Error> {
        let mut bytes = [0u8; 2];
        i2c.write_read(addr as u8, &Reg::RESULT, &mut bytes).await?;
        Ok(i16::from_be_bytes(bytes))
    }

    async fn read_config(i2c: &mut T, addr: Addr) -> Result<Config, T::Error> {
        let mut bytes = [0u8; 2];
        i2c.write_read(addr as u8, &Reg::CONFIG, &mut bytes).await?;
        Ok(u16::from_be_bytes(bytes).into())
    }

    async fn write_config(i2c: &mut T, addr: Addr, config: Config) -> Result<(), T::Error> {
        let mut bytes = [0u8; 3];
        bytes[..1].copy_from_slice(&Reg::CONFIG);
        bytes[1..].swap_with_slice(&mut u16::from(config).to_be_bytes());
        i2c.write(addr as u8, &bytes).await?;
        Ok(())
    }

    async fn write_thresh(i2c: &mut T, addr: Addr, thresh: (i16, i16)) -> Result<(), T::Error> {
        let mut bytes = [0u8; 3];
        bytes[..1].copy_from_slice(&Reg::LO_THRESH);
        bytes[1..].swap_with_slice(&mut thresh.0.to_be_bytes());
        i2c.write(addr as u8, &bytes).await?;
        bytes[..1].copy_from_slice(&Reg::HI_THRESH);
        bytes[1..].swap_with_slice(&mut thresh.1.to_be_bytes());
        i2c.write(addr as u8, &bytes).await?;
        Ok(())
    }
}

impl From<Channel> for InputMux {
    fn from(value: Channel) -> Self {
        match value {
            Channel::A0 => InputMux::A0Gnd,
            Channel::A1 => InputMux::A1Gnd,
            Channel::A2 => InputMux::A2Gnd,
            Channel::A3 => InputMux::A3Gnd,
        }
    }
}
