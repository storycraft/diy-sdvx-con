use embassy_rp::{
    Peri,
    gpio::{Level, Output, Pin},
    peripherals::*,
};
use embassy_time::Timer;

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

#[embassy_executor::task]
pub async fn led_task(cfg: LedConfig) {
    let button_1 = led(cfg.pins.button_1);
    let button_2 = led(cfg.pins.button_2);
    let button_3 = led(cfg.pins.button_3);
    let button_4 = led(cfg.pins.button_4);
    let fx_left = led(cfg.pins.fx_1);
    let fx_right = led(cfg.pins.fx_2);
    let start = led(cfg.pins.start);

    loop {
        Timer::after_secs(1).await;
    }
}

#[inline(always)]
fn led<'a>(pin: Peri<'a, impl Pin>) -> Output<'a> {
    Output::new(pin, Level::Low)
}
