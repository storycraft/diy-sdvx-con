#![no_std]
#![no_main]

mod config;
mod logger;
mod usb;

use crate::usb::usb_task;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    peripherals::USB,
    usb::{Driver as UsbDriver, InterruptHandler},
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
}
