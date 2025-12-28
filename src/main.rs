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
use embassy_executor::Spawner;
use embassy_rp::{
    adc::{self, Adc},
    bind_interrupts,
    peripherals::USB,
    usb::Driver as UsbDriver,
};

use {defmt_rtt as _, panic_halt as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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

                knob_left: p.PIN_26,
                knob_right: p.PIN_27,
            },
        },
        driver,
    );
    log::info!("USB devices initialized.");

    log::info!("Initializing LED...");
    spawner
        .spawn(led_task(LedConfig {
            pins: LedPinout {
                button_1: p.PIN_8,
                button_2: p.PIN_9,
                button_3: p.PIN_10,
                button_4: p.PIN_11,
                fx_1: p.PIN_12,
                fx_2: p.PIN_13,
                start: p.PIN_14,
            },
        }))
        .unwrap();
    log::info!("LED initialized.");

    log::info!("System started.");
    usb_task.await;
}
