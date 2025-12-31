use embassy_usb::{
    class::cdc_acm::{self, CdcAcmClass},
    driver::Driver,
};

use crate::usb;

pub fn logger_task<'a, D: Driver<'a>>(
    state: &'a mut cdc_acm::State<'a>,
    builder: &mut embassy_usb::Builder<'a, D>,
) -> impl Future<Output = ()> + use<'a, D> {
    let (sender, _) =
        CdcAcmClass::new(builder, state, usb::config::DEVICE.max_packet_size_0 as u16).split();

    defmt_embassy_usbserial::logger(sender)
}
