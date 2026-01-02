#![no_std]

use zerocopy::{FromBytes, Immutable, IntoBytes};

#[derive(Clone, Copy, Hash, PartialEq, Eq, FromBytes, IntoBytes, Immutable)]
#[repr(transparent)]
pub struct Keycode(pub u16);

include!(concat!(env!("OUT_DIR"), "/impl_keycode.rs"));
