use embassy_rp::{
    Peri,
    dma::Channel,
    flash::{self, Async, FLASH_BASE, Flash},
    peripherals::FLASH,
};
use zerocopy::{IntoBytes, TryFromBytes};

use crate::userdata::Userdata;

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

pub struct UserdataIo<'a> {
    flash: Flash<'a, FLASH, Async, { 2 * 1024 * 1024 }>,
}

impl<'a> UserdataIo<'a> {
    pub fn new(flash: Peri<'a, FLASH>, dma: Peri<'a, impl Channel>) -> Self {
        Self {
            flash: Flash::new(flash, dma),
        }
    }

    /// Perform initialization.
    pub async fn init(&mut self) -> Result<Userdata, flash::Error> {
        match self.read().await {
            Some(data) => Ok(data),
            // Saved data is invalid or failed to read
            None => {
                log::info!("Userdata is invalid. Performing initialization.");
                let userdata = Userdata::default();
                self.save(&userdata).await?;
                Ok(userdata)
            }
        }
    }

    pub async fn read(&mut self) -> Option<Userdata> {
        let mut buf = [0_u32; { size_of::<Userdata>() / 4 }];
        self.flash
            .background_read(userdata_start_offset() as _, &mut buf)
            .ok()?
            .await;

        Userdata::try_read_from_bytes(buf.as_bytes()).ok()
    }

    /// Save [`UserData`].
    /// In case of poweroff, the data may become invalid.
    /// Invalid data will be checked on startup and will be resetted.
    pub async fn save(&mut self, data: &Userdata) -> Result<(), flash::Error> {
        // Write only if changed
        if self.read().await.is_none_or(|read| read.ne(data)) {
            self.flash
                .blocking_write(userdata_start_offset() as _, data.as_bytes())?;
        }

        Ok(())
    }
}
