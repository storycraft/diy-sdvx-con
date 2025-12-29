use usbd_hid::descriptor::SerializedDescriptor;

use crate::usb::hid::{GamepadInputReport, QmkRawHidReport};

pub const DEVICE: embassy_usb::Config = device_config();

/// USB device configuration
const fn device_config() -> embassy_usb::Config<'static> {
    let mut config = embassy_usb::Config::new(0x3d5a, 0xcafe);

    config.manufacturer = Some("SDVX-Con");
    config.product = Some("SDVX Controller");

    config.max_power = 100;
    // USB 2.0 High Speed Maximum Packet Size
    config.max_packet_size_0 = 64;

    config
}

pub fn gamepad<'a>() -> embassy_usb::class::hid::Config<'a> {
    embassy_usb::class::hid::Config {
        report_descriptor: GamepadInputReport::desc(),
        request_handler: None,
        poll_ms: 1,
        max_packet_size: 8,
    }
}

pub fn via<'a>() -> embassy_usb::class::hid::Config<'a> {
    embassy_usb::class::hid::Config {
        report_descriptor: QmkRawHidReport::desc(),
        request_handler: None,
        poll_ms: 8,
        max_packet_size: 32,
    }
}
