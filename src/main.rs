#![no_std]
#![no_main]

mod input;
mod led;
mod logger;
mod usb;
mod userdata;
mod via;

use crate::{
    input::{InputConfig, InputPinout},
    led::{LedConfig, LedPinout, led_task},
    usb::usb_task,
    userdata::init_userdata,
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
use embassy_time::Timer;
use static_cell::StaticCell;

use {defmt_rtt as _, panic_halt as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Boot Phase
    let p = embassy_rp::init(Default::default());
    log::info!("System booted.");

    // System initialization phase
    log::info!("Initializing USB driver...");
    let driver = UsbDriver::new(p.USB, Irqs);
    log::info!("USB driver initialized.");

    log::info!("Initializing Adc...");
    let adc = Adc::new(p.ADC, Irqs, adc::Config::default());
    log::info!("Adc initialized.");

    // add some delay to give an attached debug probe time to parse the
    // defmt RTT header. Reading that header might touch flash memory, which
    // interferes with flash write operations.
    // https://github.com/knurling-rs/defmt/pull/683
    Timer::after_millis(10).await;

    log::info!("Initializing userdata...");
    let userdata_task = init_userdata(p.FLASH, p.DMA_CH1).await;
    spawner.must_spawn(userdata_task);
    log::info!("Userdata initialized.");

    log::info!("System initialized.");

    // Controller initialization phase
    log::info!("Initializing Controller...");

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

    log::info!("Controller started.");
    usb_task(
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
    )
    .await;
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
