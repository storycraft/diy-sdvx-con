pub mod config;
mod reader;
mod report;
mod ticker;

use embassy_executor::{SpawnToken, Spawner};
use embassy_rp::{
    Peri,
    adc::{self, Adc},
    gpio::{Input, Level, Pin, Pull},
    peripherals::*,
};
use embassy_time::Instant;

use crate::{
    input::{
        config::InputPinout,
        reader::{
            InputRead,
            button::{Button, ButtonInputReader, Buttons},
            knob::KnobInputReader,
        },
        ticker::ElapsedTimer,
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
    spawner: Spawner,
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

    spawner.must_spawn(report::gamepad_report_task(builder));
    spawner.must_spawn(report::keyboard_report_task(builder));
    spawner.must_spawn(report::media_report_task(builder));
    spawner.must_spawn(report::mouse_report_task(builder));

    task(button_reader, knob_reader)
}

#[embassy_executor::task]
async fn task(
    mut button_reader: ButtonInputReader<'static>,
    mut knob_reader: KnobInputReader<'static>,
) {
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

        report::GAMEPAD.signal(input_report(read));

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
    let mut report = GamepadInputReport::default();

    if input.buttons.button1 == Level::High {
        report.button::<7>();
    }

    if input.buttons.button2 == Level::High {
        report.button::<5>();
    }

    if input.buttons.button3 == Level::High {
        report.button::<6>();
    }

    if input.buttons.button4 == Level::High {
        report.button::<8>();
    }

    if input.buttons.fx2 == Level::High {
        report.button::<2>();
    }

    if input.buttons.start == Level::High {
        report.button::<10>();
    }

    if input.knobs.1 == KnobTurn::Left {
        report.button::<1>();
    } else if input.knobs.1 == KnobTurn::Right {
        report.button::<3>();
    }

    if input.buttons.fx1 == Level::High {
        report.dpad_down();
    }

    if input.knobs.0 == KnobTurn::Left {
        report.dpad_left();
    } else if input.knobs.0 == KnobTurn::Right {
        report.dpad_right();
    }

    report
}

#[inline(always)]
fn button<'a>(pin: Peri<'a, impl Pin>) -> Button<'a> {
    let mut input = Input::new(pin, Pull::Up);
    input.set_schmitt(true);
    input.set_inversion(true);
    Button::new(input)
}
