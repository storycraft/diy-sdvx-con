use embassy_rp::{
    Peri,
    adc::{self, Adc},
    dma,
    gpio::{Input, Level},
    peripherals::DMA_CH0,
};
use embassy_time::Instant;

use crate::input::{
    KnobTurn,
    config::{ButtonDebouncer, KnobFilter},
};

pub struct InputReader<'a> {
    last_read: Instant,

    inputs: InputDriver<'a>,

    /// ADC for knob analog conversion
    adc: Adc<'a, adc::Async>,
    /// DMA channel for ADC transfers
    dma: Peri<'a, DMA_CH0>,
    /// Knob oversample buffer
    knob_buf: KnobBuffer,
    left_knob: KnobFilter,
    right_knob: KnobFilter,
}

impl<'a> InputReader<'a> {
    pub fn new(adc: Adc<'a, adc::Async>, dma: Peri<'a, DMA_CH0>, inputs: InputDriver<'a>) -> Self {
        Self {
            last_read: Instant::MIN,

            inputs,

            adc,
            dma,
            knob_buf: KnobBuffer::new(),
            left_knob: KnobFilter::new(0),
            right_knob: KnobFilter::new(0),
        }
    }

    pub async fn read(&mut self) -> InputRead {
        let now = Instant::now();
        let duration = now.duration_since(self.last_read);
        let elapsed = duration.as_millis().min(u16::MAX as _) as u16;
        if elapsed > 0 {
            self.last_read = now;
        }

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
            left_knob: KnobTurn::from(self.left_knob.filter(left_knob, elapsed)),
            right_knob: KnobTurn::from(self.right_knob.filter(right_knob, elapsed)),
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

    pub left_knob: KnobTurn,
    pub right_knob: KnobTurn,
}

pub struct InputDriver<'a> {
    pub button1: Button<'a>,
    pub button2: Button<'a>,
    pub button3: Button<'a>,
    pub button4: Button<'a>,

    pub fx1: Button<'a>,
    pub fx2: Button<'a>,

    pub start: Button<'a>,

    pub knobs: [adc::Channel<'a>; 2],
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
) -> (u16, u16) {
    // Perform adc multi read
    adc.read_many_multichannel(knobs, &mut buf.0, 0, dma.reborrow())
        .await
        .unwrap();

    let mut knob_left = 0_u32;
    let mut knob_right = 0_u32;
    for slice in buf.0.windows(6).step_by(2) {
        let [left1, right1, left2, right2, left3, right3] = *slice else {
            unreachable!()
        };

        knob_left += median(left1, left2, left3) as u32;
        knob_right += median(right1, right2, right3) as u32;
    }

    // Average median knob value
    knob_left /= const { KNOB_SAMPLES as u32 - 2 };
    knob_right /= const { KNOB_SAMPLES as u32 - 2 };

    (knob_left as u16, knob_right as u16)
}

fn median(a: u16, b: u16, c: u16) -> u16 {
    if a >= b {
        if b >= c {
            b
        }
        // c <= b <= a
        else if a >= c {
            c
        }
        // b <= c <= a
        else {
            a
        } // b <= a <= c
    } else {
        if a >= c {
            a
        }
        // c <= a <= b
        else if b >= c {
            c
        }
        // a <= c <= b
        else {
            b
        } // a <= b <= c
    }
}
