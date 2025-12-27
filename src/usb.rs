use embassy_futures::join::join3;
use embassy_rp::{
    Peri, gpio::{Input, Pull}, peripherals::{PIN_3, USB}, usb::Driver as UsbDriver
};
use embassy_usb::class::hid::HidWriter;

use crate::{config, logger::logger_task, usb::hid::GamepadInputReport};

pub mod hid;

#[embassy_executor::task]
pub async fn usb_task(pin: Peri<'static, PIN_3>, driver: UsbDriver<'static, USB>) {
    log::info!("Initializing USB devices...");

    // Allocates descriptor and control buffer
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; { config::USB_CONFIG.max_packet_size_0 as usize }];

    // Setup function class states
    let mut hid_state = embassy_usb::class::hid::State::new();
    let mut serial_state = embassy_usb::class::cdc_acm::State::new();

    let mut builder = embassy_usb::Builder::new(
        driver,
        config::USB_CONFIG,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    // placeholder
    let gamepad_task = {
        let mut writer =
            HidWriter::<_, 8>::new(&mut builder, &mut hid_state, config::usb_gamepad_config());

        async move {
            let button = Input::new(pin, Pull::Up);
            loop {
                let report = GamepadInputReport {
                    buttons_0: 0,
                    buttons_1: 0,
                    dpad: if button.is_low() { 1 } else { 0 },
                };

                match writer.write_serialize(&report).await {
                    Ok(()) => {}
                    Err(e) => log::error!("Failed to send report: {:?}", e),
                };
            }
        }
    };

    // Setup logger task
    let logger_task = logger_task(&mut serial_state, &mut builder);
    let mut device = builder.build();

    join3(device.run(), logger_task, gamepad_task).await;
}
