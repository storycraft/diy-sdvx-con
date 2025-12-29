pub mod io;

use core::convert::Infallible;

use embassy_rp::flash::FLASH_BASE;

#[inline(always)]
/// Start address of USERDATA memory
fn userdata_start() -> usize {
    unsafe extern "C" {
        // Linker defined symbol
        static __userdata_start: u8;
    }

    &raw const __userdata_start as usize
}

#[inline]
/// Offset to start of USERDATA memory relative to FLASH memory
fn userdata_start_offset() -> usize {
    userdata_start() - FLASH_BASE as usize
}

#[inline(always)]
/// Size of USERDATA memory
fn userdata_size() -> usize {
    unsafe extern "C" {
        // Linker defined symbol
        static __userdata_size: u8;
    }

    &raw const __userdata_size as usize
}

mod layout {
    use crate::userdata::*;

    #[repr(C, align(4))]
    struct Aligned<T>(T);

    #[repr(C)]
    /// Layout reprentation in USERDATA flash
    pub struct UserdataLayout {
        pub signature: Aligned<Signature>,
        pub userdata: Aligned<UserData>,
        pub _end: Infallible,
    }
}
use layout::UserdataLayout;
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
pub struct UserData {
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
