mod reader;
mod state;

use embassy_rp::{
    Peri,
    adc::{self, Adc},
    gpio::{Input, Level, Pin, Pull},
    peripherals::*,
};
use embassy_usb::{
    class::hid::{self, HidWriter},
    driver::Driver,
};

use crate::{
    input::{
        reader::{InputDriver, InputReader},
        state::{KnobState, KnobTurn},
    },
    led::{LedState, led_sender},
    usb::{self, hid::GamepadInputReport},
};

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

    pub left_knob: Peri<'static, PIN_26>,
    pub right_knob: Peri<'static, PIN_27>,
}

#[derive(Clone, Copy)]
struct ControllerInput {
    pub button_1: Level,
    pub button_2: Level,
    pub button_3: Level,
    pub button_4: Level,
    pub fx_1: Level,
    pub fx_2: Level,
    pub start: Level,
    pub left_knob: KnobTurn,
    pub right_knob: KnobTurn,
}

pub fn input_task<'a, D: Driver<'a>>(
    cfg: InputConfig,
    state: &'a mut hid::State<'a>,
    builder: &mut embassy_usb::Builder<'a, D>,
) -> impl Future<Output = ()> + use<'a, D> {
    let mut writer = HidWriter::<_, 8>::new(builder, state, usb::config::gamepad());

    let inputs = InputDriver {
        button_1: button(cfg.pins.button_1),
        button_2: button(cfg.pins.button_2),
        button_3: button(cfg.pins.button_3),
        button_4: button(cfg.pins.button_4),
        fx_1: button(cfg.pins.fx_1),
        fx_2: button(cfg.pins.fx_2),
        start: button(cfg.pins.start),
        knobs: [
            adc::Channel::new_pin(cfg.pins.left_knob, Pull::None),
            adc::Channel::new_pin(cfg.pins.right_knob, Pull::None),
        ],
    };
    let mut reader = InputReader::new(cfg.adc, cfg.dma, inputs);

    let led_sender = led_sender();
    async move {
        writer.ready().await;

        let mut state = reader.read().await;
        let mut left_knob_state = KnobState::new(state.left_knob);
        let mut right_knob_state = KnobState::new(state.right_knob);

        loop {
            led_sender.send(LedState {
                button_1: state.button_1,
                button_2: state.button_2,
                button_3: state.button_3,
                button_4: state.button_4,
                fx_1: state.fx_1,
                fx_2: state.fx_2,
                start: state.start,
            });

            let input = ControllerInput {
                button_1: state.button_1,
                button_2: state.button_2,
                button_3: state.button_3,
                button_4: state.button_4,
                fx_1: state.fx_1,
                fx_2: state.fx_2,
                start: state.start,
                left_knob: left_knob_state.update(state.left_knob),
                right_knob: right_knob_state.update(state.right_knob),
            };

            match writer.write_serialize(&input_report(input)).await {
                Ok(()) => {}
                Err(e) => log::error!("Failed to send input report: {:?}", e),
            };

            state = reader.read().await;
        }
    }
}

#[inline(always)]
fn input_report(input: ControllerInput) -> GamepadInputReport {
    let buttons: u16 = ((input.button_1 == Level::High) as u16) << 6 // A Button (Button 7)
                | ((input.button_2 == Level::High) as u16) << 4 // B Button (Button 5)
                | ((input.button_3 == Level::High) as u16) << 5 // C Button (Button 6)
                | ((input.button_4 == Level::High) as u16) << 7 // D Button (Button 8)
                | ((input.fx_2 == Level::High) as u16) << 1 // FX Right (Button 2)
                | ((input.start == Level::High) as u16) << 9 // Start (Button 10)
                | ((input.right_knob == KnobTurn::Left) as u16) // Right knob left turn (Button 1)
                | ((input.right_knob == KnobTurn::Right) as u16) << 2; // Right knob right turn (Button 3)

    let dpad = if input.fx_1 == Level::High {
        // FX Left (Dpad down) + Left knob turns
        5 - (input.left_knob == KnobTurn::Left) as u8 + (input.left_knob == KnobTurn::Right) as u8
    } else if input.left_knob == KnobTurn::Left {
        3 // Left knob left turn (Dpad left)
    } else if input.left_knob == KnobTurn::Right {
        7 // Left knob right turn (Dpad right)
    } else {
        0
    };

    GamepadInputReport { buttons, dpad }
}

#[inline(always)]
fn button<'a>(pin: Peri<'a, impl Pin>) -> Input<'a> {
    let mut input = Input::new(pin, Pull::Up);
    input.set_schmitt(true);
    input.set_inversion(true);
    input
}
