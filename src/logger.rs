use embassy_usb::{
    class::cdc_acm::{self, CdcAcmClass},
    driver::Driver,
};

use crate::config;

pub fn logger_task<'a, D: Driver<'a>>(
    state: &'a mut cdc_acm::State<'a>,
    builder: &mut embassy_usb::Builder<'a, D>,
) -> impl Future<Output = ()> + use<'a, D> {
    let class = CdcAcmClass::new(builder, state, config::USB_CONFIG.max_packet_size_0 as u16);
    embassy_usb_logger::with_custom_style!(1024, log::LevelFilter::Info, class, |record, writer| {
        use core::fmt::Write;

        let level = record.level().as_str();
        write!(
            writer,
            "{} [{level}] {}\r\n",
            embassy_time::Instant::now(),
            record.args()
        )
        .unwrap();
    })
}
