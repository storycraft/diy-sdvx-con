use embassy_futures::join::join4;
use embassy_rp::{peripherals::USB, usb::Driver as UsbDriver};

use crate::{
    config,
    input::{InputConfig, input_task},
    logger::logger_task,
    via::via_task,
};

pub mod hid;

pub async fn usb_task(input_config: InputConfig, driver: UsbDriver<'static, USB>) {
    // Allocates descriptor and control buffer
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; { config::USB_CONFIG.max_packet_size_0 as usize }];

    // Setup function class states
    let mut hid_state = embassy_usb::class::hid::State::new();
    let mut via_state = embassy_usb::class::hid::State::new();
    let mut serial_state = embassy_usb::class::cdc_acm::State::new();

    let mut builder = embassy_usb::Builder::new(
        driver,
        config::USB_CONFIG,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    // Setup HID input task
    let input_task = input_task(input_config, &mut hid_state, &mut builder);

    // Setup via task
    let via_task = via_task(&mut via_state, &mut builder);

    // Setup logger task
    let logger_task = logger_task(&mut serial_state, &mut builder);
    let mut device = builder.build();

    join4(device.run(), logger_task, input_task, via_task).await;
}
