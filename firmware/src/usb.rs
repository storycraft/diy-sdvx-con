pub mod config;
pub mod eac;
pub mod hid;

use embassy_executor::Spawner;
use embassy_rp::{peripherals::USB, usb::Driver as UsbDriver};
use embassy_usb::{Builder, Handler, types::StringIndex};
use static_cell::{ConstStaticCell, StaticCell};

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
    let mut builder = embassy_usb::Builder::new(
        driver,
        if eac_mode {
            config::EAC_DEVICE
        } else {
            config::DEVICE
        },
        CONFIG_DESCRIPTOR.init([0; _]),
        BOS_DESCRIPTOR.init([0; _]),
        MSOS_DESCRIPTOR.init([0; _]),
        CONTROL_BUF.init([0; _]),
    );

    // Setup logger task
    spawner.must_spawn(logger_task(&mut builder));

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

    let handler = setup_handler(&mut builder);
    builder.handler(handler);

    let mut device = builder.build();
    async move { device.run().await }
}

struct UsbHandler;
impl Handler for UsbHandler {
    fn addressed(&mut self, addr: u8) {
        defmt::info!("USB Addressed: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        defmt::info!("USB Configured: {}", configured);
    }

    fn get_string(&mut self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        match index.0 {
            // EAC mode strings
            4 => Some("BT-A"),
            5 => Some("BT-B"),
            6 => Some("BT-C"),
            7 => Some("BT-D"),
            8 => Some("FX-1"),
            9 => Some("FX-2"),
            10 => Some("Start"),
            11 => Some("Controller R"),
            12 => Some("Controller G"),
            13 => Some("Controller B"),
            _ => None,
        }
    }
}

fn setup_handler(builder: &mut Builder<'static, Driver>) -> &'static mut UsbHandler {
    // Reserve EAC mode strings
    _ = builder.string(); // BT-A
    _ = builder.string(); // BT-B
    _ = builder.string(); // BT-C
    _ = builder.string(); // BT-D
    _ = builder.string(); // FX-1
    _ = builder.string(); // FX-2
    _ = builder.string(); // Start
    _ = builder.string(); // Controller R
    _ = builder.string(); // Controller G
    _ = builder.string(); // Controller B

    static HANDLER: ConstStaticCell<UsbHandler> = ConstStaticCell::new(UsbHandler);
    HANDLER.take()
}
