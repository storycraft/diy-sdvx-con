use static_cell::ConstStaticCell;
use usbd_hid::descriptor::{KeyboardReport, MouseReport, SerializedDescriptor};

use crate::usb::{
    eac::{self, EacHidHandler},
    hid::{GamepadInputReport, QmkRawHidReport},
};

pub const DEVICE: embassy_usb::Config = hid_device_config();
pub const EAC_DEVICE: embassy_usb::Config = eac_device_config();

const fn hid_device_config() -> embassy_usb::Config<'static> {
    let mut config = embassy_usb::Config::new(0x3d5a, 0xcafe);
    config.manufacturer = Some("SDVX-Con");
    config.product = Some("SDVX Controller");

    device_config(config)
}

const fn eac_device_config() -> embassy_usb::Config<'static> {
    let mut config = embassy_usb::Config::new(0x1ccf, 0x101c);
    config.manufacturer = Some("Konami Amusement");
    config.product = Some("SOUND VOLTEX controller");
    config.serial_number = Some("SDVX");

    device_config(config)
}

/// Common USB device configuration
const fn device_config(mut config: embassy_usb::Config<'static>) -> embassy_usb::Config<'static> {
    config.max_power = 100;
    // USB 2.0 High Speed Maximum Packet Size
    config.max_packet_size_0 = 64;

    config
}

pub fn eac<'a>() -> embassy_usb::class::hid::Config<'a> {
    static HANDLER: ConstStaticCell<EacHidHandler> = ConstStaticCell::new(EacHidHandler::new());

    embassy_usb::class::hid::Config {
        report_descriptor: eac::EAC_HID_DESC,
        request_handler: Some(HANDLER.take()),
        poll_ms: 1,
        max_packet_size: 8,
    }
}

pub fn gamepad<'a>() -> embassy_usb::class::hid::Config<'a> {
    embassy_usb::class::hid::Config {
        report_descriptor: GamepadInputReport::desc(),
        request_handler: None,
        poll_ms: 1,
        max_packet_size: const { size_of::<GamepadInputReport>() as u16 },
    }
}

pub fn keyboard<'a>() -> embassy_usb::class::hid::Config<'a> {
    embassy_usb::class::hid::Config {
        report_descriptor: KeyboardReport::desc(),
        request_handler: None,
        poll_ms: 1,
        max_packet_size: const { size_of::<KeyboardReport>() as u16 },
    }
}

pub fn mouse<'a>() -> embassy_usb::class::hid::Config<'a> {
    embassy_usb::class::hid::Config {
        report_descriptor: MouseReport::desc(),
        request_handler: None,
        poll_ms: 1,
        max_packet_size: const { size_of::<MouseReport>() as u16 },
    }
}

pub fn via<'a>() -> embassy_usb::class::hid::Config<'a> {
    embassy_usb::class::hid::Config {
        report_descriptor: QmkRawHidReport::desc(),
        request_handler: None,
        poll_ms: 8,
        max_packet_size: const { size_of::<QmkRawHidReport>() as u16 },
    }
}
