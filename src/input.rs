use embassy_rp::{
    Peri,
    adc::{self, Adc},
    dma,
    gpio::{Input, Level, Pin, Pull},
    peripherals::*,
};
use embassy_usb::{
    class::hid::{self, HidWriter},
    driver::Driver,
};

use crate::{
    config,
    led::{LedState, led_sender},
    usb::hid::GamepadInputReport,
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

pub struct InputButtons {
    pub button_1: Input<'static>,
    pub button_2: Input<'static>,
    pub button_3: Input<'static>,
    pub button_4: Input<'static>,

    pub fx_1: Input<'static>,
    pub fx_2: Input<'static>,

    pub start: Input<'static>,
}

#[derive(Clone, Copy)]
struct InputState {
    pub button_1: Level,
    pub button_2: Level,
    pub button_3: Level,
    pub button_4: Level,

    pub fx_1: Level,
    pub fx_2: Level,

    pub start: Level,

    pub left_knob: u8,
    pub right_knob: u8,
}

impl InputState {
    #[inline]
    pub async fn read<'a>(
        adc: &mut Adc<'a, adc::Async>,
        dma: &mut Peri<'a, impl dma::Channel>,
        buttons: &InputButtons,
        knobs: &mut [adc::Channel<'a>; 2],
        knob_buf: &mut KnobBuffer,
    ) -> Self {
        let button_1 = buttons.button_1.get_level();
        let button_2 = buttons.button_2.get_level();
        let button_3 = buttons.button_3.get_level();
        let button_4 = buttons.button_4.get_level();

        let fx_1 = buttons.fx_1.get_level();
        let fx_2 = buttons.fx_2.get_level();

        let start = buttons.start.get_level();

        let (left_knob, right_knob) = read_knob(adc, dma, knobs, knob_buf).await;

        InputState {
            button_1,
            button_2,
            button_3,
            button_4,
            fx_1,
            fx_2,
            start,
            left_knob,
            right_knob,
        }
    }
}

pub fn input_task<'a, D: Driver<'a>>(
    mut cfg: InputConfig,
    state: &'a mut hid::State<'a>,
    builder: &mut embassy_usb::Builder<'a, D>,
) -> impl Future<Output = ()> + use<'a, D> {
    let mut writer = HidWriter::<_, 8>::new(builder, state, config::usb_gamepad_config());

    let buttons = InputButtons {
        button_1: button(cfg.pins.button_1),
        button_2: button(cfg.pins.button_2),
        button_3: button(cfg.pins.button_3),
        button_4: button(cfg.pins.button_4),
        fx_1: button(cfg.pins.fx_1),
        fx_2: button(cfg.pins.fx_2),
        start: button(cfg.pins.start),
    };

    let mut knobs = [
        adc::Channel::new_pin(cfg.pins.left_knob, Pull::None),
        adc::Channel::new_pin(cfg.pins.right_knob, Pull::None),
    ];
    let mut knob_buf = KnobBuffer::new();

    let led_sender = led_sender();
    async move {
        writer.ready().await;

        let mut last_state = InputState::read(
            &mut cfg.adc,
            &mut cfg.dma,
            &buttons,
            &mut knobs,
            &mut knob_buf,
        )
        .await;
        loop {
            let state = InputState::read(
                &mut cfg.adc,
                &mut cfg.dma,
                &buttons,
                &mut knobs,
                &mut knob_buf,
            )
            .await;

            led_sender.send(LedState {
                button_1: state.button_1,
                button_2: state.button_2,
                button_3: state.button_3,
                button_4: state.button_4,
                fx_1: state.fx_1,
                fx_2: state.fx_2,
                start: state.start,
            });

            match writer
                .write_serialize(&input_report(&last_state, &state))
                .await
            {
                Ok(()) => {}
                Err(e) => log::error!("Failed to send input report: {:?}", e),
            };

            last_state = state;
        }
    }
}

#[inline(always)]
fn input_report(last_state: &InputState, state: &InputState) -> GamepadInputReport {
    let left_knob_delta = knob_delta(last_state.left_knob, state.left_knob);
    let right_knob_delta = knob_delta(last_state.right_knob, state.right_knob);

    let buttons: u16 = ((state.button_1 == Level::High) as u16) << 6 // A Button (Button 7)
                | ((state.button_2 == Level::High) as u16) << 4 // B Button (Button 5)
                | ((state.button_3 == Level::High) as u16) << 5 // C Button (Button 6)
                | ((state.button_4 == Level::High) as u16) << 7 // D Button (Button 8)
                | ((state.fx_2 == Level::High) as u16) << 1 // FX Right (Button 2)
                | ((state.start == Level::High) as u16) << 9 // Start (Button 10)
                | ((right_knob_delta < 0) as u16) // Right knob left turn (Button 1)
                | ((right_knob_delta > 0) as u16) << 2; // Right knob right turn (Button 3)
    let [buttons_0, buttons_1] = buttons.to_ne_bytes();

    let dpad = if state.fx_1 == Level::High {
        // FX Left (Dpad down) + Left knob turns
        5 - (left_knob_delta < 0) as u8 + (left_knob_delta > 0) as u8
    } else if left_knob_delta < 0 {
        3 // Right knob left turn (Dpad left)
    } else if left_knob_delta > 0 {
        7 // Right knob right turn (Dpad right)
    } else {
        0
    };

    GamepadInputReport {
        buttons_0,
        buttons_1,
        dpad,
    }
}

#[inline]
fn knob_delta(last: u8, now: u8) -> i16 {
    let d = now as i16 - last as i16;

    if d > 127 {
        d - 256
    } else if d < -127 {
        d + 256
    } else {
        d
    }
}

const KNOB_SAMPLES: usize = 32;

#[repr(transparent)]
struct KnobBuffer([u16; 2 * KNOB_SAMPLES]);

impl KnobBuffer {
    #[inline]
    pub const fn new() -> Self {
        Self([0; _])
    }
}

#[inline]
async fn read_knob<'a>(
    adc: &mut Adc<'a, adc::Async>,
    dma: &mut Peri<'a, impl dma::Channel>,
    knobs: &mut [adc::Channel<'a>; 2],
    buf: &mut KnobBuffer,
) -> (u8, u8) {
    // Perform adc multi read
    adc.read_many_multichannel(knobs, &mut buf.0, 0, dma.reborrow())
        .await
        .unwrap();

    let mut knob_left = 0_u32;
    let mut knob_right = 0_u32;
    for i in 0..KNOB_SAMPLES {
        knob_left += buf.0[i * 2] as u32;
        knob_right += buf.0[i * 2 + 1] as u32;
    }

    // Average knob value and smooth the ranges from 0 to 255
    knob_left /= const { KNOB_SAMPLES as u32 };
    knob_left >>= 4;
    knob_right /= const { KNOB_SAMPLES as u32 };
    knob_right >>= 4;

    (knob_left as u8, knob_right as u8)
}

#[inline(always)]
fn button<'a>(pin: Peri<'a, impl Pin>) -> Input<'a> {
    let mut input = Input::new(pin, Pull::Up);
    input.set_schmitt(true);
    input.set_inversion(true);
    input
}
