mod io;

use core::convert::Infallible;
use embassy_executor::SpawnToken;
use embassy_rp::{
    Peri,
    peripherals::{DMA_CH1, FLASH},
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    signal::Signal,
    watch::{Receiver, Watch},
};
use embassy_time::Timer;
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

impl Userdata {
    pub const DEFAULT: Self = Self {
        input_mode: InputMode::DEFAULT,
    };
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

impl InputMode {
    pub const DEFAULT: Self = InputMode::Gamepad;
}

static CURRENT: Watch<CriticalSectionRawMutex, Userdata, 4> = Watch::new_with(Userdata::DEFAULT);

/// Get current [`Userdata`]
pub fn get() -> Userdata {
    CURRENT.try_get().unwrap()
}

/// Set current [`Userdata`]
pub fn set(userdata: Userdata) {
    CURRENT.sender().send(userdata);
}

/// Listen for changes
pub fn listener() -> Receiver<'static, CriticalSectionRawMutex, Userdata, 4> {
    CURRENT.receiver().unwrap()
}

static SAVE_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// Request to save current [`Userdata`] to flash.
pub fn save() {
    SAVE_SIGNAL.signal(());
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
    set(userdata);

    userdata_task(io)
}

#[embassy_executor::task]
async fn userdata_task(mut io: UserdataIo<'static>) {
    loop {
        SAVE_SIGNAL.wait().await;

        match io.save(&get()).await {
            Ok(_) => {
                log::info!("Userdata saved.");
            }

            Err(e) => {
                log::error!("Failed to save userdata. error: {e:?}");
            }
        }

        // Debouncing timer
        Timer::after_secs(5).await;
    }
}
