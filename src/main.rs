#![no_std]
#![no_main]

mod config;
mod input;
mod led;
mod logger;
mod usb;

use crate::{
    input::{InputConfig, InputPinout},
    led::{LedConfig, LedPinout, led_task},
    usb::usb_task,
};
use embassy_executor::{Executor, Spawner};
use embassy_rp::{
    Peri,
    adc::{self, Adc},
    bind_interrupts,
    multicore::Stack,
    peripherals::{CORE1, USB},
    usb::Driver as UsbDriver,
};
use static_cell::StaticCell;

use {defmt_rtt as _, panic_halt as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let adc = Adc::new(p.ADC, Irqs, adc::Config::default());
    log::info!("System initialized.");

    log::info!("Initializing USB driver...");
    let driver = UsbDriver::new(p.USB, Irqs);
    log::info!("USB driver initialized.");

    log::info!("Initializing USB devices...");
    let usb_task = usb_task(
        InputConfig {
            adc,
            dma: p.DMA_CH0,
            pins: InputPinout {
                button_1: p.PIN_0,
                button_2: p.PIN_1,
                button_3: p.PIN_2,
                button_4: p.PIN_3,

                fx_1: p.PIN_4,
                fx_2: p.PIN_5,

                start: p.PIN_6,

                left_knob: p.PIN_26,
                right_knob: p.PIN_27,
            },
        },
        driver,
    );
    log::info!("USB devices initialized.");

    log::info!("Initializing Core 1...");
    start_core1(p.CORE1, |spawner| {
        log::info!("Initializing LED...");
        spawner.must_spawn(led_task(LedConfig {
            pins: LedPinout {
                button_1: p.PIN_8,
                button_2: p.PIN_9,
                button_3: p.PIN_10,
                button_4: p.PIN_11,
                fx_1: p.PIN_12,
                fx_2: p.PIN_13,
                start: p.PIN_14,
            },
        }));
        log::info!("LED initialized.");
    });
    log::info!("Core 1 initialized.");

    log::info!("System started.");
    usb_task.await;
}

fn start_core1(core1: Peri<'static, CORE1>, f: impl FnOnce(Spawner) + 'static + Send) {
    static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

    embassy_rp::multicore::spawn_core1(
        core1,
        unsafe {
            static mut CORE1_STACK: Stack<4096> = Stack::new();
            (&raw mut CORE1_STACK).as_mut().unwrap()
        },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(f);
        },
    );
}
