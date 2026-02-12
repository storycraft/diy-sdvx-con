use crate::usb::{self, Driver, hid::GamepadInputReport};
use embassy_executor::SpawnToken;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_usb::class::hid::{self, HidWriter};
use static_cell::StaticCell;
use usbd_hid::descriptor::{AsInputReport, KeyboardReport, MouseReport};

macro_rules! define_hid_task {
    ($signal:ident, $name:ident : $ty:ty, $config:expr) => {
        // Only used within input tasks.
        pub static $signal: Signal<ThreadModeRawMutex, $ty> = Signal::new();

        pub fn $name(
            builder: &mut embassy_usb::Builder<'static, Driver>,
        ) -> SpawnToken<impl Sized + use<>> {
            static STATE: StaticCell<hid::State<'static>> = StaticCell::new();

            #[embassy_executor::task]
            pub async fn inner(writer: HidWriter<'static, Driver, { size_of::<$ty>() }>) {
                task(&$signal, writer).await;
            }

            inner(HidWriter::new(
                builder,
                STATE.init(hid::State::new()),
                $config,
            ))
        }
    };
}

#[inline]
async fn task<T: AsInputReport, const N: usize>(
    rx: &Signal<ThreadModeRawMutex, T>,
    mut writer: HidWriter<'static, Driver, N>,
) -> ! {
    writer.ready().await;

    loop {
        match writer.write_serialize(&rx.wait().await).await {
            Ok(()) => {}
            Err(e) => defmt::error!("Failed to send input report: {:?}", e),
        };
    }
}

define_hid_task!(GAMEPAD, gamepad_report_task: GamepadInputReport, usb::config::gamepad());
define_hid_task!(KEYBOARD, keyboard_report_task: KeyboardReport, usb::config::keyboard());
define_hid_task!(MOUSE, mouse_report_task: MouseReport, usb::config::mouse());
