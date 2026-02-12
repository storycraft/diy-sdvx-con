mod builder;
pub mod config;
mod key;
pub mod reader;
mod report;
mod ticker;

use core::cell::Cell;

use embassy_executor::{SpawnToken, Spawner};
use embassy_futures::join::join;
use embassy_rp::gpio::Level;
use embassy_sync::blocking_mutex::{NoopMutex, ThreadModeMutex, raw::NoopRawMutex};
use embassy_time::Instant;

use crate::{
    input::{
        key::InputReports,
        reader::{InputRead, button::ButtonInputReader, knob::KnobInputReader},
        ticker::ElapsedTimer,
    },
    led::{self, LedState},
    usb::Driver,
    userdata::{self, keymap::Keymap},
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum KnobTurn {
    #[default]
    None,
    Left,
    Right,
}

impl KnobTurn {
    pub const DEFAULT: Self = Self::None;
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
    button_reader: ButtonInputReader<'static>,
    knob_reader: KnobInputReader<'static>,
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

    spawner.must_spawn(report::gamepad_report_task(builder));
    spawner.must_spawn(report::keyboard_report_task(builder));
    spawner.must_spawn(report::mouse_report_task(builder));

    inner(button_reader, knob_reader)
}

pub static CURRENT_INPUT: ThreadModeMutex<Cell<InputRead>> =
    ThreadModeMutex::new(Cell::new(InputRead::DEFAULT));

async fn input_updater(
    mut button_reader: ButtonInputReader<'static>,
    mut knob_reader: KnobInputReader<'static>,
    keymap: &NoopMutex<Keymap>,
) {
    CURRENT_INPUT.borrow().set(InputRead {
        knobs: knob_reader.read(0).await,
        buttons: button_reader.read(0),
    });

    let mut ticker = ElapsedTimer::new(Instant::now());
    loop {
        let read = CURRENT_INPUT.borrow().get();
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

        loop {
            let elapsed_ms = ticker.next_elapsed_ms();
            let next = InputRead {
                knobs: knob_reader.read(elapsed_ms).await,
                buttons: button_reader.read(elapsed_ms),
            };

            if next != read || next != InputRead::DEFAULT {
                CURRENT_INPUT.borrow().set(next);
                break;
            }
        }
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

    reports.key(keymap.button1, input.buttons.button1 == Level::High);
    reports.key(keymap.button2, input.buttons.button2 == Level::High);
    reports.key(keymap.button3, input.buttons.button3 == Level::High);
    reports.key(keymap.button4, input.buttons.button4 == Level::High);
    reports.key(keymap.fx1, input.buttons.fx1 == Level::High);
    reports.key(keymap.fx2, input.buttons.fx2 == Level::High);
    reports.key(keymap.start, input.buttons.start == Level::High);

    reports.key(keymap.left_knob_left, input.knobs.0 == KnobTurn::Left);
    reports.key(keymap.left_knob_right, input.knobs.0 == KnobTurn::Right);

    reports.key(keymap.right_knob_left, input.knobs.1 == KnobTurn::Left);
    reports.key(keymap.right_knob_right, input.knobs.1 == KnobTurn::Right);

    reports.send();
}
