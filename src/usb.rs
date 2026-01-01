use embassy_executor::Spawner;
use embassy_rp::{peripherals::USB, usb::Driver as UsbDriver};
use static_cell::StaticCell;

use crate::{
    input::{InputConfig, input_task},
    logger::logger_task,
    via::via_task,
};

pub mod config;
pub mod hid;

pub type Driver = UsbDriver<'static, USB>;

pub fn init_usb(
    spawner: Spawner,
    input_config: InputConfig,
    driver: Driver,
) -> impl Future + 'static {
    // Allocates descriptor and control buffer
    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; config::DEVICE.max_packet_size_0 as usize]> =
        StaticCell::new();

    let mut builder = embassy_usb::Builder::new(
        driver,
        config::DEVICE,
        CONFIG_DESCRIPTOR.init([0; _]),
        BOS_DESCRIPTOR.init([0; _]),
        MSOS_DESCRIPTOR.init([0; _]),
        CONTROL_BUF.init([0; _]),
    );

    // Setup HID input task
    spawner.must_spawn(input_task(input_config, &mut builder));

    // Setup via task
    spawner.must_spawn(via_task(&mut builder));

    // Setup logger task
    spawner.must_spawn(logger_task(&mut builder));

    let mut device = builder.build();
    async move { device.run().await }
}
