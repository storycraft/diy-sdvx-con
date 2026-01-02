pub mod config;
mod reader;
mod ticker;
mod writer;

use embassy_executor::SpawnToken;
use embassy_rp::{
    Peri,
    adc::{self, Adc},
    gpio::{Input, Level, Pin, Pull},
    peripherals::*,
};
use embassy_time::Instant;
use embassy_usb::class::hid::State;
use static_cell::StaticCell;
use zerocopy::little_endian;

use crate::{
    input::{
        config::InputPinout,
        reader::{
            InputRead,
            button::{Button, ButtonInputReader, Buttons},
            knob::KnobInputReader,
        },
        ticker::ElapsedTimer,
        writer::HidInputWriter,
    },
    led::{self, LedState},
    usb::{Driver, hid::GamepadInputReport},
};

pub struct InputConfig {
    /// ADC for knob analog conversion
    pub adc: Adc<'static, adc::Async>,

    /// DMA channel for ADC transfers
    pub dma: Peri<'static, DMA_CH0>,

    /// Button and knob pinout
    pub pins: InputPinout<'static>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum KnobTurn {
    #[default]
    None,
    Left,
    Right,
}

impl From<i16> for KnobTurn {
    fn from(value: i16) -> Self {
        match value {
            0 => Self::None,
            ..0 => Self::Left,
            1.. => Self::Right,
        }
    }
}

pub fn input_task(
    cfg: InputConfig,
    builder: &mut embassy_usb::Builder<'static, Driver>,
) -> SpawnToken<impl Sized + use<>> {
    let button_reader = ButtonInputReader::new(Buttons {
        button1: button(cfg.pins.button1),
        button2: button(cfg.pins.button2),
        button3: button(cfg.pins.button3),
        button4: button(cfg.pins.button4),
        fx1: button(cfg.pins.fx1),
        fx2: button(cfg.pins.fx2),
        start: button(cfg.pins.start),
    });
    let knob_reader = KnobInputReader::new(
        [
            adc::Channel::new_pin(cfg.pins.left_knob, Pull::None),
            adc::Channel::new_pin(cfg.pins.right_knob, Pull::None),
        ],
        cfg.adc,
        cfg.dma,
    );

    let writer = HidInputWriter::new(builder, {
        static STATES: StaticCell<[State; 4]> = StaticCell::new();
        STATES.init([const { State::new() }; 4])
    });

    task(button_reader, knob_reader, writer)
}

#[embassy_executor::task]
async fn task(
    mut button_reader: ButtonInputReader<'static>,
    mut knob_reader: KnobInputReader<'static>,
    mut writer: HidInputWriter,
) {
    writer.ready().await;

    let mut ticker = ElapsedTimer::new(Instant::now());
    let mut read = InputRead {
        knobs: knob_reader.read(0).await,
        buttons: button_reader.read(0),
    };
    loop {
        led::update(LedState {
            button_1: read.buttons.button1,
            button_2: read.buttons.button2,
            button_3: read.buttons.button3,
            button_4: read.buttons.button4,
            fx_1: read.buttons.fx1,
            fx_2: read.buttons.fx2,
            start: read.buttons.start,
        });

        match writer.gamepad.write_serialize(&input_report(read)).await {
            Ok(()) => {}
            Err(e) => defmt::error!("Failed to send input report: {:?}", e),
        };

        loop {
            let elapsed_ms = ticker.next_elapsed_ms();
            let next_read = InputRead {
                knobs: knob_reader.read(elapsed_ms).await,
                buttons: button_reader.read(elapsed_ms),
            };

            if next_read != read {
                read = next_read;
                break;
            }
        }
    }
}

#[inline(always)]
fn input_report(input: InputRead) -> GamepadInputReport {
    let buttons: u16 = ((input.buttons.button1 == Level::High) as u16) << 6 // A Button (Button 7)
                | ((input.buttons.button2 == Level::High) as u16) << 4 // B Button (Button 5)
                | ((input.buttons.button3 == Level::High) as u16) << 5 // C Button (Button 6)
                | ((input.buttons.button4 == Level::High) as u16) << 7 // D Button (Button 8)
                | ((input.buttons.fx2 == Level::High) as u16) << 1 // FX Right (Button 2)
                | ((input.buttons.start == Level::High) as u16) << 9 // Start (Button 10)
                | ((input.knobs.1 == KnobTurn::Left) as u16) // Right knob left turn (Button 1)
                | ((input.knobs.1 == KnobTurn::Right) as u16) << 2; // Right knob right turn (Button 3)

    let dpad = if input.buttons.fx1 == Level::High {
        // FX Left (Dpad down) + Left knob turns
        5 + (input.knobs.0 == KnobTurn::Left) as u8 - (input.knobs.0 == KnobTurn::Right) as u8
    } else if input.knobs.0 == KnobTurn::Left {
        7 // Left knob left turn (Dpad left)
    } else if input.knobs.0 == KnobTurn::Right {
        3 // Left knob right turn (Dpad right)
    } else {
        0
    };

    GamepadInputReport {
        buttons: little_endian::U16::new(buttons),
        dpad,
    }
}

#[inline(always)]
fn button<'a>(pin: Peri<'a, impl Pin>) -> Button<'a> {
    let mut input = Input::new(pin, Pull::Up);
    input.set_schmitt(true);
    input.set_inversion(true);
    Button::new(input)
}
