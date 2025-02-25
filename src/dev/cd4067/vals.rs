use bilge::prelude::*;
use defmt::Format;

#[bitsize(4)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, Format)]
pub enum Channel {
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
    C10,
    C11,
    C12,
    C13,
    C14,
    C15,
}
