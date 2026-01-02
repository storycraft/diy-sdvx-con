pub mod config;
mod key;
mod reader;
mod report;
mod ticker;

use embassy_executor::{SpawnToken, Spawner};
use embassy_futures::join::join;
use embassy_rp::{
    Peri,
    adc::{self, Adc},
    gpio::{Input, Level, Pin, Pull},
    peripherals::*,
};
use embassy_sync::blocking_mutex::{NoopMutex, raw::NoopRawMutex};
use embassy_time::Instant;
use keycode::Keycode;
use usbd_hid::descriptor::{KeyboardReport, MediaKeyboardReport, MouseReport};

use crate::{
    input::{
        config::InputPinout,
        key::InputReports,
        reader::{
            InputRead,
            button::{Button, ButtonInputReader, Buttons},
            knob::KnobInputReader,
        },
        ticker::ElapsedTimer,
    },
    led::{self, LedState},
    usb::{Driver, hid::GamepadInputReport},
    userdata::{self, keymap::Keymap},
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
    #[embassy_executor::task]
    async fn inner(
        button_reader: ButtonInputReader<'static>,
        knob_reader: KnobInputReader<'static>,
    ) {
        let keymap = NoopMutex::const_new(
            NoopRawMutex::new(),
            userdata::get(|userdata| userdata.keymap.clone()),
        );

        join(
            input_updater(button_reader, knob_reader, &keymap),
            keymap_updater(&keymap),
        )
        .await;
    }

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

    inner(button_reader, knob_reader)
}

#[inline(always)]
fn button<'a>(pin: Peri<'a, impl Pin>) -> Button<'a> {
    let mut input = Input::new(pin, Pull::Up);
    input.set_schmitt(true);
    input.set_inversion(true);
    Button::new(input)
}

async fn input_updater(
    mut button_reader: ButtonInputReader<'static>,
    mut knob_reader: KnobInputReader<'static>,
    keymap: &NoopMutex<Keymap>,
) {
    let mut read = InputRead {
        knobs: knob_reader.read(0).await,
        buttons: button_reader.read(0),
    };
    let mut ticker = ElapsedTimer::new(Instant::now());
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

        keymap.lock(|keymap| {
            report_inputs(keymap, read);
        });

        let elapsed_ms = ticker.next_elapsed_ms();
        read = InputRead {
            knobs: knob_reader.read(elapsed_ms).await,
            buttons: button_reader.read(elapsed_ms),
        };
    }
}

async fn keymap_updater(keymap: &NoopMutex<Keymap>) {
    let mut listener = userdata::listener();
    loop {
        listener.changed().await;

        let new_keymap = userdata::get(|userdata| userdata.keymap.clone());
        unsafe {
            keymap.lock_mut(|keymap| {
                *keymap = new_keymap;
            });
        }
    }
}

fn report_inputs(keymap: &Keymap, input: InputRead) {
    let mut reports = InputReports::default();

    if input.buttons.button1 == Level::High {
        reports.press_key(keymap.button1);
    }

    if input.buttons.button2 == Level::High {
        reports.press_key(keymap.button2);
    }

    if input.buttons.button3 == Level::High {
        reports.press_key(keymap.button3);
    }

    if input.buttons.button4 == Level::High {
        reports.press_key(keymap.button4);
    }

    if input.buttons.fx1 == Level::High {
        reports.press_key(keymap.fx1);
    }

    if input.buttons.fx2 == Level::High {
        reports.press_key(keymap.fx2);
    }

    if input.buttons.start == Level::High {
        reports.press_key(keymap.start);
    }

    if input.knobs.0 == KnobTurn::Left {
        reports.press_key(keymap.left_knob_left);
    } else if input.knobs.0 == KnobTurn::Right {
        reports.press_key(keymap.left_knob_right);
    }

    if input.knobs.1 == KnobTurn::Left {
        reports.press_key(keymap.right_knob_left);
    } else if input.knobs.1 == KnobTurn::Right {
        reports.press_key(keymap.right_knob_right);
    }

    reports.send();
}
