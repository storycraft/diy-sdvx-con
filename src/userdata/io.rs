use core::mem::offset_of;

use embassy_rp::{
    Peri,
    dma::Channel,
    flash::{self, Async, FLASH_BASE, Flash},
    peripherals::FLASH,
};
use zerocopy::{FromZeros, IntoBytes, TryFromBytes};

use crate::userdata::{Signature, Userdata};

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

use crate::userdata::*;

#[repr(C, align(4))]
struct Aligned<T>(T);

#[repr(C)]
/// Layout reprentation in USERDATA flash
struct UserdataLayout {
    pub signature: Aligned<Signature>,
    pub userdata: Aligned<Userdata>,
    pub _end: Infallible,
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

    /// Check if userdata is valid
    pub async fn is_valid(&mut self) -> Result<bool, flash::Error> {
        let mut signature = Signature::new_zeroed();
        self.flash
            .read(
                userdata_offset(offset_of!(UserdataLayout, signature)),
                signature.as_mut_bytes(),
            )
            .await?;

        Ok(signature == Signature::CURRENT)
    }

    /// Perform initialization.
    pub async fn init(&mut self) -> Result<Userdata, flash::Error> {
        if !self.is_valid().await? {
            log::info!("Userdata is invalid. Performing initialization.");
            let userdata = Userdata::default();
            self.save(&userdata).await?;
            return Ok(userdata);
        }

        match self.read().await {
            Some(data) => Ok(data),
            // Saved data is invalid or failed to read
            None => {
                let userdata = Userdata::default();
                self.save(&userdata).await?;
                Ok(userdata)
            }
        }
    }

    pub async fn read(&mut self) -> Option<Userdata> {
        let mut buf = [0_u8; { size_of::<Userdata>() }];
        self.flash
            .read(
                userdata_offset(offset_of!(UserdataLayout, userdata)),
                &mut buf,
            )
            .await
            .ok()?;

        Userdata::try_read_from_bytes(&buf).ok()
    }

    /// Save [`UserData`].
    /// In case of poweroff, the data may become invalid.
    /// Invalid data will be checked on startup and will be resetted.
    pub async fn save(&mut self, data: &Userdata) -> Result<(), flash::Error> {
        // Write only if changed
        if self.read().await.is_none_or(|read| read.ne(data)) {
            self.flash.blocking_write(
                userdata_offset(offset_of!(UserdataLayout, userdata)),
                data.as_bytes(),
            )?;
        }

        if !self.is_valid().await? {
            // Set signature
            self.flash.blocking_write(
                userdata_offset(offset_of!(UserdataLayout, signature)),
                Signature::CURRENT.as_bytes(),
            )?;
        }

        Ok(())
    }
}

fn userdata_offset(offset: usize) -> u32 {
    const _: () = const {
        // Check if it's always safe to convert
        assert!(size_of::<usize>() == size_of::<u32>());
    };

    (userdata_start_offset() + offset) as u32
}
