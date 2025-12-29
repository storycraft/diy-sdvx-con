mod io;

use core::{cell::RefCell, convert::Infallible};
use embassy_executor::SpawnToken;
use embassy_rp::{
    Peri,
    peripherals::{DMA_CH1, FLASH},
};
use embassy_sync::{
    blocking_mutex::{Mutex, raw::CriticalSectionRawMutex},
    signal::Signal,
    watch::{Receiver, Watch},
};
use embassy_time::Timer;
use scopeguard::defer;
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
    Gamepad = 0,
    /// Controller uses configurable hid input
    Keyboard = 1,
}

impl InputMode {
    pub const DEFAULT: Self = InputMode::Gamepad;

    pub fn to_num(self) -> u8 {
        self as u8
    }

    pub fn from_num(v: u8) -> Option<Self> {
        match v {
            0 => Some(InputMode::Gamepad),
            1 => Some(InputMode::Keyboard),
            _ => None,
        }
    }
}

static CURRENT: Mutex<CriticalSectionRawMutex, RefCell<Userdata>> =
    Mutex::new(RefCell::new(Userdata::DEFAULT));
static WATCH: Watch<CriticalSectionRawMutex, (), 4> = Watch::new();

#[inline]
/// Get current [`Userdata`]
pub fn get<R>(f: impl FnOnce(&Userdata) -> R) -> R {
    CURRENT.lock(|cell| f(&cell.borrow()))
}

#[inline]
/// Update current [`Userdata`]
pub fn update<R>(f: impl FnOnce(&mut Userdata) -> R) -> R {
    defer!(WATCH.sender().send(()));
    CURRENT.lock(|cell| f(&mut cell.borrow_mut()))
}

/// Listen for changes
pub fn listener() -> Receiver<'static, CriticalSectionRawMutex, (), 4> {
    WATCH.receiver().unwrap()
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
    CURRENT.lock(|cell| {
        *cell.borrow_mut() = userdata;
    });

    userdata_task(io)
}

#[embassy_executor::task]
async fn userdata_task(mut io: UserdataIo<'static>) {
    loop {
        SAVE_SIGNAL.wait().await;

        match io.save(&get(|userdata| *userdata)).await {
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
