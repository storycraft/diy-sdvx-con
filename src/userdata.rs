mod io;

use core::convert::Infallible;
use zerocopy::{FromBytes, Immutable, IntoBytes, TryFromBytes};

/// Magic number for identifying if [`UserData`] in flash is valid or not.
#[derive(Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable)]
#[repr(transparent)]
struct Signature(u32);

impl Signature {
    /// Current signature.
    /// Change on every [`UserData`] changes.
    pub const CURRENT: Self = Signature(0x26d67ba0);

    pub fn new(sig: u32) -> Self {
        Self(sig)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, TryFromBytes, IntoBytes, Immutable)]
pub struct Userdata {
    pub input_mode: InputMode,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, TryFromBytes, IntoBytes, Immutable)]
#[repr(u8)]
pub enum InputMode {
    /// Controller uses fixed Gamepad input
    #[default]
    Gamepad,
    /// Controller uses configurable hid input
    Keyboard,
}
