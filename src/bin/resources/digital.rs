use pumpedli::dev::cd4067::Channel;
use pumpedli::mux;

pub const OUTPUTS: [mux::Output; 16] = [
    mux::Output(Channel::C0),
    mux::Output(Channel::C1),
    mux::Output(Channel::C2),
    mux::Output(Channel::C3),
    mux::Output(Channel::C4),
    mux::Output(Channel::C5),
    mux::Output(Channel::C6),
    mux::Output(Channel::C7),
    mux::Output(Channel::C8),
    mux::Output(Channel::C9),
    mux::Output(Channel::C10),
    mux::Output(Channel::C11),
    mux::Output(Channel::C12),
    mux::Output(Channel::C13),
    mux::Output(Channel::C14),
    mux::Output(Channel::C15),
];
