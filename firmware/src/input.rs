mod config;
mod debouncer;
mod reader;
mod state;

use embassy_executor::SpawnToken;
use embassy_futures::join::join4;
use embassy_rp::{
    Peri,
    adc::{self, Adc},
    gpio::{Input, Level, Pin, Pull},
    peripherals::*,
};
use embassy_time::{Duration, Ticker};
use embassy_usb::class::hid::{self, HidWriter, State};
use static_cell::StaticCell;
use usbd_hid::descriptor::{KeyboardReport, MediaKeyboardReport, MouseReport};
use zerocopy::native_endian;

use crate::{
    input::{
        reader::{DebouncedInput, InputDriver, InputReader},
        state::{KnobState, KnobTurn},
    },
    led::{self, LedState},
    usb::{self, Driver, hid::GamepadInputReport},
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
    pub button1: Peri<'static, PIN_0>,
    pub button2: Peri<'static, PIN_1>,
    pub button3: Peri<'static, PIN_2>,
    pub button4: Peri<'static, PIN_3>,

    pub fx1: Peri<'static, PIN_4>,
    pub fx2: Peri<'static, PIN_5>,

    pub start: Peri<'static, PIN_6>,

    pub left_knob: Peri<'static, PIN_26>,
    pub right_knob: Peri<'static, PIN_27>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ControllerInput {
    pub button1: Level,
    pub button2: Level,
    pub button3: Level,
    pub button4: Level,
    pub fx1: Level,
    pub fx2: Level,
    pub start: Level,
    pub left_knob: KnobTurn,
    pub right_knob: KnobTurn,
}

impl ControllerInput {
    pub const NONE: Self = Self {
        button1: Level::Low,
        button2: Level::Low,
        button3: Level::Low,
        button4: Level::Low,
        fx1: Level::Low,
        fx2: Level::Low,
        start: Level::Low,
        left_knob: KnobTurn::None,
        right_knob: KnobTurn::None,
    };
}

pub fn input_task(
    cfg: InputConfig,
    builder: &mut embassy_usb::Builder<'static, Driver>,
) -> SpawnToken<impl Sized + use<>> {
    #[embassy_executor::task]
    async fn inner(mut reader: InputReader<'static>, mut writer: HidInputWriter) {
        writer.ready().await;

        let mut state = reader.read().await;
        let mut left_knob_state = KnobState::new(state.left_knob);
        let mut right_knob_state = KnobState::new(state.right_knob);

        let mut ticker = Ticker::every(Duration::from_millis(5));
        let mut last_input = ControllerInput::NONE;
        loop {
            let input = ControllerInput {
                button1: state.button1,
                button2: state.button2,
                button3: state.button3,
                button4: state.button4,
                fx1: state.fx1,
                fx2: state.fx2,
                start: state.start,
                left_knob: left_knob_state.update(state.left_knob),
                right_knob: right_knob_state.update(state.right_knob),
            };

            if last_input != input {
                last_input = input;
                led::update(LedState {
                    button_1: state.button1,
                    button_2: state.button2,
                    button_3: state.button3,
                    button_4: state.button4,
                    fx_1: state.fx1,
                    fx_2: state.fx2,
                    start: state.start,
                });

                match writer.gamepad.write_serialize(&input_report(input)).await {
                    Ok(()) => {}
                    Err(e) => defmt::error!("Failed to send input report: {:?}", e),
                };

                // TODO:: remove global input ticker
                ticker.next().await;
            }

            state = reader.read().await;
        }
    }

    let inputs = InputDriver {
        button1: button(cfg.pins.button1),
        button2: button(cfg.pins.button2),
        button3: button(cfg.pins.button3),
        button4: button(cfg.pins.button4),
        fx1: button(cfg.pins.fx1),
        fx2: button(cfg.pins.fx2),
        start: button(cfg.pins.start),
        knobs: [
            adc::Channel::new_pin(cfg.pins.left_knob, Pull::None),
            adc::Channel::new_pin(cfg.pins.right_knob, Pull::None),
        ],
    };
    let reader = InputReader::new(cfg.adc, cfg.dma, inputs);

    let writer = HidInputWriter::new(builder, {
        static STATES: StaticCell<[State; 4]> = StaticCell::new();
        STATES.init([const { State::new() }; 4])
    });

    inner(reader, writer)
}

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

    async fn ready(&mut self) {
        join4(
            self.gamepad.ready(),
            self.keyboard.ready(),
            self.mouse.ready(),
            self.media.ready(),
        )
        .await;
    }
}

#[inline(always)]
fn input_report(input: ControllerInput) -> GamepadInputReport {
    let buttons: u16 = ((input.button1 == Level::High) as u16) << 6 // A Button (Button 7)
                | ((input.button2 == Level::High) as u16) << 4 // B Button (Button 5)
                | ((input.button3 == Level::High) as u16) << 5 // C Button (Button 6)
                | ((input.button4 == Level::High) as u16) << 7 // D Button (Button 8)
                | ((input.fx2 == Level::High) as u16) << 1 // FX Right (Button 2)
                | ((input.start == Level::High) as u16) << 9 // Start (Button 10)
                | ((input.right_knob == KnobTurn::Left) as u16) // Right knob left turn (Button 1)
                | ((input.right_knob == KnobTurn::Right) as u16) << 2; // Right knob right turn (Button 3)

    let dpad = if input.fx1 == Level::High {
        // FX Left (Dpad down) + Left knob turns
        5 + (input.left_knob == KnobTurn::Left) as u8 - (input.left_knob == KnobTurn::Right) as u8
    } else if input.left_knob == KnobTurn::Left {
        7 // Left knob left turn (Dpad left)
    } else if input.left_knob == KnobTurn::Right {
        3 // Left knob right turn (Dpad right)
    } else {
        0
    };

    GamepadInputReport { buttons: little_endian::U16::new(buttons), dpad }
}

#[inline(always)]
fn button<'a>(pin: Peri<'a, impl Pin>) -> DebouncedInput<'a> {
    let mut input = Input::new(pin, Pull::Up);
    input.set_schmitt(true);
    input.set_inversion(true);
    DebouncedInput::new(input)
}
