mod io;

use core::convert::Infallible;
use embassy_executor::SpawnToken;
use embassy_rp::{
    Peri,
    peripherals::{DMA_CH1, FLASH},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, TryFromBytes};

use crate::userdata::io::UserdataIo;

/// Magic number for identifying if [`UserData`] in flash is valid or not.
#[derive(Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, Immutable)]
#[repr(transparent)]
struct Signature(u32);

impl Signature {
    /// Current signature.
    /// Change on every [`UserData`] changes.
    pub const CURRENT: Self = Signature(0x26d67ba0);
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

pub async fn init_userdata(
    flash: Peri<'static, FLASH>,
    dma: Peri<'static, DMA_CH1>,
) -> SpawnToken<impl Sized> {
    let mut io = UserdataIo::new(flash, dma);

    let userdata = match io.init().await {
        Ok(data) => data,
        Err(e) => {
            log::error!("Userdata initialization failed error: {e:?}. Fallback to default.");
            Userdata::default()
        }
    };
    userdata_task(io, userdata)
}

#[embassy_executor::task]
async fn userdata_task(io: UserdataIo<'static>, mut userdata: Userdata) {}
