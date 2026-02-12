#![no_std]
#![no_main]

mod input;
mod keycodes;
mod led;
mod logger;
mod usb;
mod userdata;
mod via;

use crate::{
    input::{
        config::InputPinout,
        reader::{
            button::{self, ButtonInputReader},
            knob::KnobInputReader,
        },
    },
    led::{LedConfig, LedPinout, led_task},
    usb::init_usb,
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

use {defmt_embassy_usbserial as _, panic_halt as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Boot Phase
    let p = embassy_rp::init(Default::default());
    defmt::info!("System booted.");

    // System initialization phase
    defmt::info!("Initializing USB driver...");
    let driver = UsbDriver::new(p.USB, Irqs);
    defmt::info!("USB driver initialized.");

    defmt::info!("Initializing Adc...");
    let adc = Adc::new(p.ADC, Irqs, adc::Config::default());
    defmt::info!("Adc initialized.");

    // add some delay to give an attached debug probe time to parse the
    // defmt RTT header. Reading that header might touch flash memory, which
    // interferes with flash write operations.
    // https://github.com/knurling-rs/defmt/pull/683
    Timer::after_millis(10).await;

    defmt::info!("Initializing userdata...");
    let userdata_task = init_userdata(p.FLASH, p.DMA_CH1).await;
    spawner.must_spawn(userdata_task);
    defmt::info!("Userdata initialized.");

    defmt::info!("System initialized.");

    // Controller initialization phase
    defmt::info!("Initializing Controller...");

    defmt::info!("Initializing input...");
    let (buttons, knobs) = InputPinout {
        button1: p.PIN_0,
        button2: p.PIN_1,
        button3: p.PIN_2,
        button4: p.PIN_3,

        fx1: p.PIN_4,
        fx2: p.PIN_5,

        start: p.PIN_6,

        left_knob: p.PIN_26,
        right_knob: p.PIN_27,
    }
    .inputs();

    read_mode_hotkey(&buttons);

    let button_reader = ButtonInputReader::new(buttons);
    let knob_reader = KnobInputReader::new(knobs, adc, p.DMA_CH0);
    defmt::info!("Input initialized.");

    defmt::info!("Initializing USB...");
    let usb_task = init_usb(spawner, button_reader, knob_reader, driver);
    defmt::info!("USB Initialized.");

    defmt::info!("Initializing Core 1...");
    start_core1(p.CORE1, |spawner| {
        defmt::info!("Initializing LED...");
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
        defmt::info!("LED initialized.");
    });
    defmt::info!("Core 1 initialized.");

    defmt::info!("Controller started.");
    usb_task.await;
}

fn read_mode_hotkey(button: &button::Buttons) {
    if !button.start.input.is_high() {
        return;
    }
    let prev_eac_mode = userdata::get(|data| data.eac_mode);

    // Start + Button 1: Enable EAC mode
    if button.button1.input.is_high() && !prev_eac_mode {
        userdata::update(|data| {
            data.eac_mode = true;
        });
        userdata::save();

        defmt::info!("EAC mode enabled via hotkey.");

    // Start + Button 2: Enable HID mode
    } else if button.button2.input.is_high() && prev_eac_mode {
        userdata::update(|data| {
            data.eac_mode = false;
        });
        userdata::save();

        defmt::info!("HID mode enabled via hotkey.");
    }
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
