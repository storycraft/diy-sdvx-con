use embassy_rp::{
    Peri,
    adc::{self, Adc},
    peripherals::DMA_CH0,
};

use crate::input::config::{KNOB_SAMPLES, KnobFilter};

pub struct KnobInputReader<'a> {
    /// ADC for knob analog conversion
    adc: Adc<'a, adc::Async>,
    /// DMA channel for ADC transfers
    dma: Peri<'a, DMA_CH0>,

    knobs: [adc::Channel<'a>; 2],
    left_filter: KnobFilter,
    right_filter: KnobFilter,

    /// Knob oversample buffer
    knob_buf: [u16; 2 * KNOB_SAMPLES],
}

impl<'a> KnobInputReader<'a> {
    pub fn new(
        knobs: [adc::Channel<'a>; 2],
        adc: Adc<'a, adc::Async>,
        dma: Peri<'a, DMA_CH0>,
    ) -> Self {
        Self {
            adc,
            dma,

            knobs,
            left_filter: KnobFilter::new(0),
            right_filter: KnobFilter::new(0),

            knob_buf: [0; _],
        }
    }

    pub async fn read(&mut self, elapsed_ms: u16) -> (i16, i16) {
        // Perform adc multi read
        self.adc
            .read_many_multichannel(&mut self.knobs, &mut self.knob_buf, 96, self.dma.reborrow())
            .await
            .unwrap();

        let mut knob_left = 4096_u16;
        let mut knob_right = 4096_u16;
        let (chunks, []) = self.knob_buf.as_chunks::<2>() else {
            unreachable!();
        };
        for slice in chunks.windows(3) {
            let [[left1, right1], [left2, right2], [left3, right3]] = *slice else {
                unreachable!()
            };

            knob_left = knob_left.min(median(left1, left2, left3));
            knob_right = knob_right.min(median(right1, right2, right3));
        }

        (
            self.left_filter.filter(knob_left as _, elapsed_ms),
            self.right_filter.filter(knob_right as _, elapsed_ms),
        )
    }
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
    } else if a >= c {
        a
    }
    // c <= a <= b
    else if b >= c {
        c
    }
    // a <= c <= b
    else {
        b
    }
}
