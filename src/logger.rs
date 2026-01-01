use embassy_executor::SpawnToken;
use embassy_usb::class::cdc_acm::{CdcAcmClass, Sender, State};
use static_cell::StaticCell;

use crate::usb::{self, Driver};

pub fn logger_task(
    builder: &mut embassy_usb::Builder<'static, Driver>,
) -> SpawnToken<impl Sized + use<>> {
    #[embassy_executor::task]
    async fn inner(sender: Sender<'static, Driver>) {
        defmt_embassy_usbserial::logger(sender).await;
    }

    let (sender, _) = CdcAcmClass::new(
        builder,
        {
            static STATE: StaticCell<State> = StaticCell::new();
            STATE.init(State::new())
        },
        usb::config::DEVICE.max_packet_size_0 as u16,
    )
    .split();
    inner(sender)
}
