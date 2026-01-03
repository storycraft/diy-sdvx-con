use embassy_rp::gpio::{Input, Level};

use crate::input::config::ButtonDebouncer;

pub struct ButtonInputReader<'a> {
    inputs: Buttons<'a>,
}

impl<'a> ButtonInputReader<'a> {
    pub const fn new(inputs: Buttons<'a>) -> Self {
        Self { inputs }
    }

    pub fn read(&mut self, elapsed_ms: u16) -> ButtonInputRead {
        let button1 = self.inputs.button1.read(elapsed_ms);
        let button2 = self.inputs.button2.read(elapsed_ms);
        let button3 = self.inputs.button3.read(elapsed_ms);
        let button4 = self.inputs.button4.read(elapsed_ms);

        let fx1 = self.inputs.fx1.read(elapsed_ms);
        let fx2 = self.inputs.fx2.read(elapsed_ms);

        let start = self.inputs.start.read(elapsed_ms);

        ButtonInputRead {
            button1,
            button2,
            button3,
            button4,
            fx1,
            fx2,
            start,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ButtonInputRead {
    pub button1: Level,
    pub button2: Level,
    pub button3: Level,
    pub button4: Level,

    pub fx1: Level,
    pub fx2: Level,

    pub start: Level,
}

impl ButtonInputRead {
    pub const DEFAULT: Self = Self {
        button1: Level::Low,
        button2: Level::Low,
        button3: Level::Low,
        button4: Level::Low,
        fx1: Level::Low,
        fx2: Level::Low,
        start: Level::Low,
    };
}

impl Default for ButtonInputRead {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub struct Buttons<'a> {
    pub button1: Button<'a>,
    pub button2: Button<'a>,
    pub button3: Button<'a>,
    pub button4: Button<'a>,

    pub fx1: Button<'a>,
    pub fx2: Button<'a>,

    pub start: Button<'a>,
}

pub struct Button<'a> {
    input: Input<'a>,
    debouncer: ButtonDebouncer,
}

impl<'a> Button<'a> {
    pub const fn new(input: Input<'a>) -> Self {
        Self {
            input,
            debouncer: ButtonDebouncer::new(false),
        }
    }

    fn read(&mut self, elapsed_ms: u16) -> Level {
        Level::from(self.debouncer.debounce(self.input.is_high(), elapsed_ms))
    }
}
