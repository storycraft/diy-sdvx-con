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
    usb::{Driver, eac::EacInputReport},
    userdata::{self, keymap::Keymap},
};

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

pub fn eac_input_task(
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
        input_read_loop(button_reader, knob_reader, report_eac_inputs).await;
    }

    spawner.must_spawn(report::eac_report_task(builder));
    inner(button_reader, knob_reader)
}

pub fn hid_input_task(
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

        let hid_input_updater = input_read_loop(button_reader, knob_reader, |read| {
            keymap.lock(|keymap| {
                report_hid_inputs(keymap, read);

                led::update(LedState {
                    button_1: read.buttons.button1,
                    button_2: read.buttons.button2,
                    button_3: read.buttons.button3,
                    button_4: read.buttons.button4,
                    fx_1: read.buttons.fx1,
                    fx_2: read.buttons.fx2,
                    start: read.buttons.start,
                });
            });
        });

        join(hid_input_updater, keymap_updater(&keymap)).await;
    }

    spawner.must_spawn(report::gamepad_report_task(builder));
    spawner.must_spawn(report::keyboard_report_task(builder));
    spawner.must_spawn(report::mouse_report_task(builder));
    inner(button_reader, knob_reader)
}

pub static CURRENT_INPUT: ThreadModeMutex<Cell<InputRead>> =
    ThreadModeMutex::new(Cell::new(InputRead::DEFAULT));

async fn input_read_loop(
    mut button_reader: ButtonInputReader<'static>,
    mut knob_reader: KnobInputReader<'static>,
    mut f: impl FnMut(InputRead),
) {
    CURRENT_INPUT.borrow().set(InputRead {
        knobs: knob_reader.read(0).await,
        buttons: button_reader.read(0),
    });

    let mut ticker = ElapsedTimer::new(Instant::now());
    loop {
        let read = CURRENT_INPUT.borrow().get();
        f(read);

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

fn report_eac_inputs(input: InputRead) {
    report::EAC.signal(EacInputReport {
        report_id: 4,
        buttons: input.buttons.button1 as u16
            | ((input.buttons.button2 as u16) << 1)
            | ((input.buttons.button3 as u16) << 2)
            | ((input.buttons.button4 as u16) << 3)
            | ((input.buttons.fx1 as u16) << 4)
            | ((input.buttons.fx2 as u16) << 5)
            | ((input.buttons.start as u16) << 8),
        x: (input.knobs.0 >> 5) as i8,
        y: (input.knobs.1 >> 5) as i8,
    });
}

fn report_hid_inputs(keymap: &Keymap, input: InputRead) {
    let mut reports = InputReports::default();

    reports.key(keymap.button1, input.buttons.button1 == Level::High);
    reports.key(keymap.button2, input.buttons.button2 == Level::High);
    reports.key(keymap.button3, input.buttons.button3 == Level::High);
    reports.key(keymap.button4, input.buttons.button4 == Level::High);
    reports.key(keymap.fx1, input.buttons.fx1 == Level::High);
    reports.key(keymap.fx2, input.buttons.fx2 == Level::High);
    reports.key(keymap.start, input.buttons.start == Level::High);

    let left_knob = KnobTurn::from(input.knobs.0);
    reports.key(keymap.left_knob_left, left_knob == KnobTurn::Left);
    reports.key(keymap.left_knob_right, left_knob == KnobTurn::Right);

    let right_knob = KnobTurn::from(input.knobs.1);
    reports.key(keymap.right_knob_left, right_knob == KnobTurn::Left);
    reports.key(keymap.right_knob_right, right_knob == KnobTurn::Right);

    reports.send();
}
