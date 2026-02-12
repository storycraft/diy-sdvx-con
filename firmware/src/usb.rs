pub mod config;
pub mod eac;
pub mod hid;

use embassy_executor::Spawner;
use embassy_rp::{peripherals::USB, usb::Driver as UsbDriver};
use static_cell::StaticCell;

use crate::{
    input::{
        eac_input_task, hid_input_task,
        reader::{button::ButtonInputReader, knob::KnobInputReader},
    },
    logger::logger_task,
    userdata,
    via::via_task,
};

pub type Driver = UsbDriver<'static, USB>;

pub fn init_usb(
    spawner: Spawner,
    button_reader: ButtonInputReader<'static>,
    knob_reader: KnobInputReader<'static>,
    driver: Driver,
) -> impl Future + 'static {
    // Allocates descriptor and control buffer
    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; config::DEVICE.max_packet_size_0 as usize]> =
        StaticCell::new();

    let eac_mode = userdata::get(|data| data.eac_mode);
    let config = if eac_mode {
        config::EAC_DEVICE
    } else {
        config::DEVICE
    };
    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        CONFIG_DESCRIPTOR.init([0; _]),
        BOS_DESCRIPTOR.init([0; _]),
        MSOS_DESCRIPTOR.init([0; _]),
        CONTROL_BUF.init([0; _]),
    );

    // Setup logger task
    spawner.must_spawn(logger_task(&mut builder));

    let eac_mode = userdata::get(|data| data.eac_mode);
    if eac_mode {
        // Setup EAC input task
        spawner.must_spawn(eac_input_task(
            spawner,
            button_reader,
            knob_reader,
            &mut builder,
        ));
    } else {
        // Setup HID input task
        spawner.must_spawn(hid_input_task(
            spawner,
            button_reader,
            knob_reader,
            &mut builder,
        ));

        // Setup via task
        spawner.must_spawn(via_task(&mut builder));
    }

    let mut device = builder.build();
    async move { device.run().await }
}
