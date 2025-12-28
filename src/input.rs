use embassy_rp::{
    Peri,
    adc::{self, Adc},
    dma,
    gpio::{Input, Pin, Pull},
    peripherals::*,
};
use embassy_usb::{
    class::hid::{self, HidWriter},
    driver::Driver,
};

use crate::{config, usb::hid::GamepadInputReport};

pub struct InputConfig {
    /// ADC for knob analog conversion
    pub adc: Adc<'static, adc::Async>,

    /// DMA channel for ADC transfers
    pub dma: Peri<'static, DMA_CH0>,

    /// Button and knob pinout
    pub pins: InputPinout,
}

pub struct InputPinout {
    pub button_1: Peri<'static, PIN_0>,
    pub button_2: Peri<'static, PIN_1>,
    pub button_3: Peri<'static, PIN_2>,
    pub button_4: Peri<'static, PIN_3>,

    pub fx_1: Peri<'static, PIN_4>,
    pub fx_2: Peri<'static, PIN_5>,

    pub start: Peri<'static, PIN_6>,

    pub knob_left: Peri<'static, PIN_26>,
    pub knob_right: Peri<'static, PIN_27>,
}

pub fn input_task<'a, D: Driver<'a>>(
    cfg: InputConfig,
    state: &'a mut hid::State<'a>,
    builder: &mut embassy_usb::Builder<'a, D>,
) -> impl Future<Output = ()> + use<'a, D> {
    let mut writer = HidWriter::<_, 8>::new(builder, state, config::usb_gamepad_config());

    let button_1 = button(cfg.pins.button_1);
    let button_2 = button(cfg.pins.button_2);
    let button_3 = button(cfg.pins.button_3);
    let button_4 = button(cfg.pins.button_4);
    let fx_left = button(cfg.pins.fx_1);
    let fx_right = button(cfg.pins.fx_2);
    let start = button(cfg.pins.start);

    let knob_left = adc::Channel::new_pin(cfg.pins.knob_left, Pull::None);
    let knob_right = adc::Channel::new_pin(cfg.pins.knob_right, Pull::None);
    let knobs = [knob_left, knob_right];
    let mut knob_inputs = [0u16; 2];

    async move {
        writer.ready().await;

        loop {
            let buttons: u16 = (button_1.is_high() as u16) << 7 // A Button (Button 7)
                | (button_2.is_high() as u16) << 5 // B Button (Button 5)
                | (button_3.is_high() as u16) << 6 // C Button (Button 6)
                | (button_4.is_high() as u16) << 8 // D Button (Button 8)
                | (fx_right.is_high() as u16) << 2 // FX Right (Button 2)
                | (start.is_high() as u16) << 1; // Start (Button 1)

            let [buttons_0, buttons_1] = buttons.to_be_bytes();
            match writer
                .write_serialize(&GamepadInputReport {
                    buttons_0,
                    buttons_1,
                    dpad: 0,
                })
                .await
            {
                Ok(()) => {}
                Err(e) => log::error!("Failed to send report: {:?}", e),
            };
        }
    }
}

async fn knob_task<'a>(
    mut adc: Adc<'static, adc::Async>,
    mut knobs: [adc::Channel<'a>; 2],
    knob_inputs: &mut [u16; 2],
    mut dma: Peri<'a, impl dma::Channel>,
) {
    loop {
        adc.read_many_multichannel(&mut knobs, knob_inputs, 1, dma.reborrow())
            .await
            .unwrap();
    }
}

#[inline(always)]
fn button<'a>(pin: Peri<'a, impl Pin>) -> Input<'a> {
    let mut input = Input::new(pin, Pull::Up);
    input.set_schmitt(true);
    input
}
