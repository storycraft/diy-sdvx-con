use embassy_rp::{
    Peri,
    adc::{self, Adc},
    dma,
    gpio::{Input, Level},
    peripherals::DMA_CH0,
};
use embassy_time::{Duration, Instant};

use crate::input::{config::DEBOUNCE_MS, debouncer::ButtonDebouncer};

pub struct InputReader<'a> {
    last_read: Instant,

    inputs: InputDriver<'a>,

    /// ADC for knob analog conversion
    adc: Adc<'a, adc::Async>,
    /// DMA channel for ADC transfers
    dma: Peri<'a, DMA_CH0>,
    /// Knob oversample buffer
    knob_buf: KnobBuffer,
}

impl<'a> InputReader<'a> {
    pub fn new(adc: Adc<'a, adc::Async>, dma: Peri<'a, DMA_CH0>, inputs: InputDriver<'a>) -> Self {
        Self {
            last_read: Instant::MIN,

            inputs,

            adc,
            dma,
            knob_buf: KnobBuffer::new(),
        }
    }

    pub async fn read(&mut self) -> InputRead {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_read);
        self.last_read = now;

        let button1 = self.inputs.button1.read(elapsed);
        let button2 = self.inputs.button2.read(elapsed);
        let button3 = self.inputs.button3.read(elapsed);
        let button4 = self.inputs.button4.read(elapsed);

        let fx1 = self.inputs.fx1.read(elapsed);
        let fx2 = self.inputs.fx2.read(elapsed);

        let start = self.inputs.start.read(elapsed);

        let (left_knob, right_knob) = read_knob(
            &mut self.adc,
            &mut self.dma,
            &mut self.inputs.knobs,
            &mut self.knob_buf,
        )
        .await;

        InputRead {
            button1,
            button2,
            button3,
            button4,
            fx1,
            fx2,
            start,
            left_knob,
            right_knob,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InputRead {
    pub button1: Level,
    pub button2: Level,
    pub button3: Level,
    pub button4: Level,

    pub fx1: Level,
    pub fx2: Level,

    pub start: Level,

    pub left_knob: u16,
    pub right_knob: u16,
}

pub struct InputDriver<'a> {
    pub button1: DebouncedInput<'a>,
    pub button2: DebouncedInput<'a>,
    pub button3: DebouncedInput<'a>,
    pub button4: DebouncedInput<'a>,

    pub fx1: DebouncedInput<'a>,
    pub fx2: DebouncedInput<'a>,

    pub start: DebouncedInput<'a>,

    pub knobs: [adc::Channel<'a>; 2],
}

pub struct DebouncedInput<'a> {
    input: Input<'a>,
    debouncer: ButtonDebouncer<DEBOUNCE_MS>,
}

impl<'a> DebouncedInput<'a> {
    pub const fn new(input: Input<'a>) -> Self {
        Self {
            input,
            debouncer: ButtonDebouncer::new(false),
        }
    }

    fn read(&mut self, elapsed: Duration) -> Level {
        Level::from(self.debouncer.debounce(self.input.is_high(), elapsed))
    }
}

const KNOB_SAMPLES: usize = 256;

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
) -> (u16, u16) {
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

    // Average knob value
    knob_left /= const { KNOB_SAMPLES as u32 };
    knob_right /= const { KNOB_SAMPLES as u32 };

    (knob_left as u16, knob_right as u16)
}
