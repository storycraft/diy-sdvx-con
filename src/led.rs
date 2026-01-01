use embassy_futures::join::join;
use embassy_rp::{
    Peri,
    gpio::{Level, Output, Pin},
    peripherals::*,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use embassy_time::{Duration, Ticker};

pub struct LedConfig {
    /// LED pinout
    pub pins: LedPinout,
}

pub struct LedPinout {
    pub button_1: Peri<'static, PIN_8>,
    pub button_2: Peri<'static, PIN_9>,
    pub button_3: Peri<'static, PIN_10>,
    pub button_4: Peri<'static, PIN_11>,

    pub fx_1: Peri<'static, PIN_12>,
    pub fx_2: Peri<'static, PIN_13>,

    pub start: Peri<'static, PIN_14>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LedState {
    pub button_1: Level,
    pub button_2: Level,
    pub button_3: Level,
    pub button_4: Level,

    pub fx_1: Level,
    pub fx_2: Level,

    pub start: Level,
}

impl Default for LedState {
    fn default() -> Self {
        Self {
            button_1: Level::Low,
            button_2: Level::Low,
            button_3: Level::Low,
            button_4: Level::Low,
            fx_1: Level::Low,
            fx_2: Level::Low,
            start: Level::Low,
        }
    }
}

static LED_STATE: Signal<CriticalSectionRawMutex, LedState> = Signal::new();

#[inline]
pub fn update(led: LedState) {
    LED_STATE.signal(led);
}

#[embassy_executor::task]
pub async fn led_task(cfg: LedConfig) {
    let mut button_1 = led(cfg.pins.button_1);
    let mut button_2 = led(cfg.pins.button_2);
    let mut button_3 = led(cfg.pins.button_3);
    let mut button_4 = led(cfg.pins.button_4);
    let mut fx_1 = led(cfg.pins.fx_1);
    let mut fx_2 = led(cfg.pins.fx_2);
    let mut start = led(cfg.pins.start);

    let mut last_state = LedState::default();
    let mut ticker = Ticker::every(Duration::from_millis(8));
    loop {
        // Limit updates maximum 125Hz
        let (state, _) = join(LED_STATE.wait(), ticker.next()).await;
        if state == last_state {
            continue;
        }

        button_1.set_level(state.button_1);
        button_2.set_level(state.button_2);
        button_3.set_level(state.button_3);
        button_4.set_level(state.button_4);
        fx_1.set_level(state.fx_1);
        fx_2.set_level(state.fx_2);
        start.set_level(state.start);

        last_state = state;
    }
}

#[inline(always)]
fn led<'a>(pin: Peri<'a, impl Pin>) -> Output<'a> {
    Output::new(pin, Level::Low)
}
