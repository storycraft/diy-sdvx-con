use embassy_futures::join::join4;
use embassy_usb::class::hid::{self, HidWriter};
use usbd_hid::descriptor::{KeyboardReport, MediaKeyboardReport, MouseReport};

use crate::usb::{self, Driver, hid::GamepadInputReport};

pub struct HidInputWriter {
    pub gamepad: HidWriter<'static, Driver, { size_of::<GamepadInputReport>() }>,
    pub keyboard: HidWriter<'static, Driver, { size_of::<KeyboardReport>() }>,
    pub mouse: HidWriter<'static, Driver, { size_of::<MouseReport>() }>,
    pub media: HidWriter<'static, Driver, { size_of::<MediaKeyboardReport>() }>,
}

impl HidInputWriter {
    pub fn new(
        builder: &mut embassy_usb::Builder<'static, Driver>,
        states: &'static mut [hid::State<'static>; 4],
    ) -> Self {
        let [gamepad_state, keyboard_state, mouse_state, media_state] = states;

        Self {
            gamepad: HidWriter::new(builder, gamepad_state, usb::config::gamepad()),
            keyboard: HidWriter::new(builder, keyboard_state, usb::config::keyboard()),
            mouse: HidWriter::new(builder, mouse_state, usb::config::mouse()),
            media: HidWriter::new(builder, media_state, usb::config::media_control()),
        }
    }

    pub async fn ready(&mut self) {
        join4(
            self.gamepad.ready(),
            self.keyboard.ready(),
            self.mouse.ready(),
            self.media.ready(),
        )
        .await;
    }
}
