mod io;
pub mod keymap;

use core::cell::RefCell;
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
use embassy_time::{Duration, Ticker};
use scopeguard::defer;
use zerocopy::{Immutable, IntoBytes, TryFromBytes};

use crate::userdata::{io::UserdataIo, keymap::Keymap};

/// Magic number for identifying if [`UserData`] in flash is valid or not.
#[derive(Clone, Copy, PartialEq, Eq, TryFromBytes, IntoBytes, Immutable)]
#[repr(u32)]
pub enum Signature {
    /// Current signature.
    /// Change on every [`UserData`] changes.
    Current = 0x26d67ba1,
}

#[derive(Clone, PartialEq, Eq, TryFromBytes, IntoBytes, Immutable)]
#[repr(C)]
pub struct Userdata {
    pub signature: Signature,
    pub keymap: Keymap,
}

impl Userdata {
    pub const DEFAULT: Self = Self {
        signature: Signature::Current,
        keymap: Keymap::DEFAULT,
    };
}

impl Default for Userdata {
    fn default() -> Self {
        Self::DEFAULT
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
    let mut ticker = Ticker::every(Duration::from_secs(5));
    loop {
        SAVE_SIGNAL.wait().await;

        match io.save(&get(|userdata| userdata.clone())).await {
            Ok(_) => {
                log::info!("Userdata saved.");
            }

            Err(e) => {
                log::error!("Failed to save userdata. error: {e:?}");
            }
        }

        // Debouncing timer
        ticker.next().await;
    }
}
