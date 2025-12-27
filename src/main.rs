#![no_std]
#![no_main]

mod config;
mod logger;

use crate::logger::setup_logger_task;
use embassy_executor::Spawner;
use embassy_futures::join::join3;
use embassy_rp::{
    bind_interrupts,
    peripherals::USB,
    usb::{Driver as UsbDriver, InterruptHandler},
};
use embassy_usb::class::{
    cdc_acm::{self},
    hid::{self, HidWriter},
};
use usbd_hid::descriptor::KeyboardReport;

use {defmt_rtt as _, panic_halt as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    log::info!("System initialized.");

    log::info!("Setting up USB driver...");
    let driver = UsbDriver::new(p.USB, Irqs);
    log::info!("USB driver initialized.");

    _ = spawner.spawn(usb_task(driver));

    loop {
        // log::info!("Hello world!");
        embassy_futures::yield_now().await;
    }
}

#[embassy_executor::task]
async fn usb_task(driver: UsbDriver<'static, USB>) {
    log::debug!("Initializing USB devices...");

    // Allocates descriptor and control buffer
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; { config::USB_CONFIG.max_packet_size_0 as usize }];

    // Setup function class states
    let mut hid_state = hid::State::new();
    let mut serial_state = cdc_acm::State::new();

    let mut builder = embassy_usb::Builder::new(
        driver,
        config::USB_CONFIG,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    // placeholder
    let keyboard_task = {
        let mut writer =
            HidWriter::<_, 8>::new(&mut builder, &mut hid_state, config::usb_gamepad_config());

        async move {
            loop {
                let report = KeyboardReport {
                    keycodes: [4, 0, 0, 0, 0, 0],
                    leds: 0,
                    modifier: 0,
                    reserved: 0,
                };

                match writer.write_serialize(&report).await {
                    Ok(()) => {}
                    Err(e) => log::error!("Failed to send report: {:?}", e),
                };
            }
        }
    };

    // Setup logger task
    let logger_task = setup_logger_task(&mut serial_state, &mut builder);
    let mut device = builder.build();

    log::info!("Starting USB devices...");
    join3(device.run(), logger_task, keyboard_task).await;
}
