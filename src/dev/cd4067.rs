mod vals;

use bilge::prelude::*;
use embassy_rp::gpio::Output;

pub use vals::Channel;

pub struct Cd4067<'a> {
    en: Output<'a>,
    s0: Output<'a>,
    s1: Output<'a>,
    s2: Output<'a>,
    s3: Output<'a>,
}

impl<'a> Cd4067<'a> {
    pub fn new(
        en: Output<'a>,
        s0: Output<'a>,
        s1: Output<'a>,
        s2: Output<'a>,
        s3: Output<'a>,
    ) -> Self {
        Self { en, s0, s1, s2, s3 }
    }

    pub fn enable(&mut self, channel: Channel) {
        self.disable();

        #[bitsize(4)]
        #[derive(FromBits)]
        struct ChannelBits(bool, bool, bool, bool);

        let bits = ChannelBits::from(u4::from(channel));
        let bits = [bits.val_0(), bits.val_1(), bits.val_2(), bits.val_3()];
        let [s0, s1, s2, s3] = bits.map(Into::into);

        self.s0.set_level(s0);
        self.s1.set_level(s1);
        self.s2.set_level(s2);
        self.s3.set_level(s3);

        self.en.set_low();
    }

    pub fn disable(&mut self) {
        self.en.set_high();
    }
}
