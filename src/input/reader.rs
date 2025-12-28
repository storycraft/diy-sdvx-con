use embassy_rp::{
    Peri,
    adc::{self, Adc},
    dma,
    gpio::{Input, Level},
    peripherals::DMA_CH0,
};

pub struct InputReader<'a> {
    inputs: ControllerInputs<'a>,

    /// ADC for knob analog conversion
    adc: Adc<'a, adc::Async>,
    /// DMA channel for ADC transfers
    dma: Peri<'a, DMA_CH0>,
    /// Knob oversample buffer
    knob_buf: KnobBuffer,
}

impl<'a> InputReader<'a> {
    pub fn new(
        adc: Adc<'a, adc::Async>,
        dma: Peri<'a, DMA_CH0>,
        inputs: ControllerInputs<'a>,
    ) -> Self {
        Self {
            inputs,

            adc,
            dma,
            knob_buf: KnobBuffer::new(),
        }
    }

    pub async fn read(&mut self) -> InputRead {
        let button_1 = self.inputs.button_1.get_level();
        let button_2 = self.inputs.button_2.get_level();
        let button_3 = self.inputs.button_3.get_level();
        let button_4 = self.inputs.button_4.get_level();

        let fx_1 = self.inputs.fx_1.get_level();
        let fx_2 = self.inputs.fx_2.get_level();

        let start = self.inputs.start.get_level();

        let (left_knob, right_knob) = read_knob(
            &mut self.adc,
            &mut self.dma,
            &mut self.inputs.knobs,
            &mut self.knob_buf,
        )
        .await;

        InputRead {
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

pub struct ControllerInputs<'a> {
    pub button_1: Input<'a>,
    pub button_2: Input<'a>,
    pub button_3: Input<'a>,
    pub button_4: Input<'a>,

    pub fx_1: Input<'a>,
    pub fx_2: Input<'a>,

    pub start: Input<'a>,

    pub knobs: [adc::Channel<'a>; 2],
}

#[derive(Clone, Copy)]
pub struct InputRead {
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
