#![no_std]

use zerocopy::{FromBytes, Immutable, IntoBytes};

#[derive(Clone, Copy, Hash, PartialEq, Eq, FromBytes, IntoBytes, Immutable)]
#[repr(transparent)]
pub struct Keycode(pub u16);

impl From<u16> for Keycode {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl Default for Keycode {
    fn default() -> Self {
        Self::KC_NO
    }
}

include!(concat!(env!("OUT_DIR"), "/impl_keycode.rs"));
