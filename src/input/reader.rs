use embassy_rp::{
    Peri,
    adc::{self, Adc},
    dma,
    gpio::{Input, Level},
    peripherals::DMA_CH0,
};

pub struct InputReader<'a> {
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
            inputs,

            adc,
            dma,
            knob_buf: KnobBuffer::new(),
        }
    }

    pub async fn read(&mut self) -> InputRead {
        let button1 = self.inputs.button1.get_level();
        let button2 = self.inputs.button2.get_level();
        let button3 = self.inputs.button3.get_level();
        let button4 = self.inputs.button4.get_level();

        let fx1 = self.inputs.fx1.get_level();
        let fx2 = self.inputs.fx2.get_level();

        let start = self.inputs.start.get_level();

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

pub struct InputDriver<'a> {
    pub button1: Input<'a>,
    pub button2: Input<'a>,
    pub button3: Input<'a>,
    pub button4: Input<'a>,

    pub fx1: Input<'a>,
    pub fx2: Input<'a>,

    pub start: Input<'a>,

    pub knobs: [adc::Channel<'a>; 2],
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

    pub left_knob: u8,
    pub right_knob: u8,
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
