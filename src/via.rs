use embassy_usb::{
    class::hid::{self, HidReaderWriter},
    driver::Driver,
};

use crate::{config, usb::hid::QmkRawHidReport};

pub fn via_task<'a, D: Driver<'a>>(
    state: &'a mut hid::State<'a>,
    builder: &mut embassy_usb::Builder<'a, D>,
) -> impl Future<Output = ()> + use<'a, D> {
    let mut io = HidReaderWriter::<_, 32, 32>::new(builder, state, config::usb_via_config());
    async move {
        io.ready().await;
        let (reader, mut writer) = io.split();

        loop {
            log::info!("Reporting rawhid");

            match writer
                .write_serialize(&QmkRawHidReport { data: [0; 32] })
                .await
            {
                Ok(()) => {}
                Err(e) => log::error!("Failed to send rawhid report: {:?}", e),
            };
        }
    }
}
