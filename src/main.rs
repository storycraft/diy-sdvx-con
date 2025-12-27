#![no_std]
#![no_main]

mod config;
mod gamepad;
mod logger;

use crate::{gamepad::GamepadInputReport, logger::setup_logger_task};
use embassy_executor::Spawner;
use embassy_futures::join::join3;
use embassy_rp::{
    Peri, bind_interrupts,
    gpio::{Input, Pull},
    peripherals::{PIN_3, USB},
    usb::{Driver as UsbDriver, InterruptHandler},
};
use embassy_usb::class::{
    cdc_acm::{self},
    hid::{self, HidWriter},
};

use {defmt_rtt as _, panic_halt as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    log::info!("System initialized.");

    log::info!("Initializing USB driver...");
    let driver = UsbDriver::new(p.USB, Irqs);
    log::info!("USB driver initialized.");

    let usb_task = usb_task(p.PIN_3, driver);
    log::info!("USB devices initialized.");
    _ = spawner.spawn(usb_task);

    log::info!("System started.");
    loop {
        // log::info!("Hello world!");
        embassy_futures::yield_now().await;
    }
}

#[embassy_executor::task]
async fn usb_task(pin: Peri<'static, PIN_3>, driver: UsbDriver<'static, USB>) {
    log::info!("Initializing USB devices...");

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
            let button = Input::new(pin, Pull::Up);
            loop {
                log::info!("button: {}", button.is_low());
                let report = GamepadInputReport {
                    buttons_0: 0,
                    buttons_1: 0,
                    dpad: if button.is_low() { 2 } else { 0 } ,
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

    join3(device.run(), logger_task, keyboard_task).await;
}
