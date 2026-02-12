use usbd_hid::descriptor::generator_prelude::*;
use zerocopy::FromBytes;

#[rustfmt::skip]
pub const EAC_HID_DESC: &[u8] = &[
    0x05, 0x01, //      Usage Page (Generic Desktop Ctrls)
    0x09, 0x04, //      Usage (Joystick)
    0xA1, 0x01, //      Collection (Application)

    // HID Input
    0x85, 0x04, //          Report ID (4)
    // Buttons (9 bits)
    0x05, 0x09, //          Usage Page (Button)
    0x19, 0x01, //          Usage Minimum (0x01)
    0x29, 0x09, //          Usage Maximum (0x09)
    0x15, 0x00, //          Logical Minimum (0)
    0x25, 0x01, //          Logical Maximum (1)
    0x75, 0x01, //          Report Size (1)
    0x95, 0x10, //          Report Count (9)
    0x81, 0x02, //          Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // 7 bits padding
    0x75, 0x07, //          Report Size (7)
    0x95, 0x01, //          Report Count (1)
    0x81, 0x03, //          Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // Analog axis (2 bytes)
    0x05, 0x01, //          Usage Page (Generic Desktop Ctrls)
    0x09, 0x01, //          Usage (Pointer)
    0x15, 0x00, //          Logical Minimum (0)
    0x26, 0xFF, 0x00, //    Logical Maximum (255)
    0x75, 0x08, //          Report Size (8)
    0x95, 0x02, //          Report Count (2)
    0xA1, 0x00, //          Collection (Physical)
    0x09, 0x30, //              Usage (X)
    0x09, 0x31, //              Usage (Y)
    0x81, 0x02, //              Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0, //                End Collection (analog axis)

    // LED Output
    0x85, 0x05, //          Report ID (5)
    0x15, 0x00, //          Logical Minimum (0)
    0x25, 0x01, //          Logical Maximum (1)
    // LED 1
    0x05, 0x0a, //          Usage Page (Vendor Defined 0x0A)
    0x09, 0x01, //          Usage (0x01)
    0xa1, 0x02, //          Collection (Logical)
    0x05, 0x08, //              Usage Page (LEDs)
    0x09, 0x4b, //              Usage (Generic Indicator 1)
    0x79, 0x04, //              String Index (4)
    0x75, 0x01, //              Report Size (1)
    0x95, 0x01, //              Report Count (1)
    0x91, 0x02, //              Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xc0, //                End Collection
    // LED 2
    0x05, 0x0a, //          Usage Page (Vendor Defined 0x0A)
    0x09, 0x02, //          Usage (0x02)
    0xa1, 0x02, //          Collection (Logical)
    0x05, 0x08, //              Usage Page (LEDs)
    0x09, 0x4b, //              Usage (Generic Indicator 1)
    0x79, 0x05, //              String Index (5)
    0x75, 0x01, //              Report Size (1)
    0x95, 0x01, //              Report Count (1)
    0x91, 0x02, //              Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xc0, //                End Collection
    // LED 3
    0x05, 0x0a, //          Usage Page (Vendor Defined 0x0A)
    0x09, 0x03, //          Usage (0x03)
    0xa1, 0x02, //          Collection (Logical)
    0x05, 0x08, //              Usage Page (LEDs)
    0x09, 0x4b, //              Usage (Generic Indicator 1)
    0x79, 0x06, //              String Index (6)
    0x75, 0x01, //              Report Size (1)
    0x95, 0x01, //              Report Count (1)
    0x91, 0x02, //              Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xc0, //                End Collection
    // LED 4
    0x05, 0x0a, //          Usage Page (Vendor Defined 0x0A)
    0x09, 0x04, //          Usage (0x04)
    0xa1, 0x02, //          Collection (Logical)
    0x05, 0x08, //              Usage Page (LEDs)
    0x09, 0x4b, //              Usage (Generic Indicator 1)
    0x79, 0x07, //              String Index (7)
    0x75, 0x01, //              Report Size (1)
    0x95, 0x01, //              Report Count (1)
    0x91, 0x02, //              Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xc0, //                End Collection
    // LED FX-1
    0x05, 0x0a, //          Usage Page (Vendor Defined 0x0A)
    0x09, 0x05, //          Usage (0x05)
    0xa1, 0x02, //          Collection (Logical)
    0x05, 0x08, //              Usage Page (LEDs)
    0x09, 0x4b, //              Usage (Generic Indicator 1)
    0x79, 0x08, //              String Index (8)
    0x75, 0x01, //              Report Size (1)
    0x95, 0x01, //              Report Count (1)
    0x91, 0x02, //              Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xc0, //                End Collection
    // LED FX-2
    0x05, 0x0a, //          Usage Page (Vendor Defined 0x0A)
    0x09, 0x06, //          Usage (0x06)
    0xa1, 0x02, //          Collection (Logical)
    0x05, 0x08, //              Usage Page (LEDs)
    0x09, 0x4b, //              Usage (Generic Indicator 1)
    0x79, 0x09, //              String Index (9)
    0x75, 0x01, //              Report Size (1)
    0x95, 0x01, //              Report Count (1)
    0x91, 0x02, //              Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xc0, //                End Collection
    // LED Start
    0x05, 0x0a, //          Usage Page (Vendor Defined 0x0A)
    0x09, 0x07, //          Usage (0x07)
    0xa1, 0x02, //          Collection (Logical)
    0x05, 0x08, //              Usage Page (LEDs)
    0x09, 0x4b, //              Usage (Generic Indicator 1)
    0x79, 0x0a, //              String Index (10)
    0x75, 0x01, //              Report Size (1)
    0x95, 0x01, //              Report Count (1)
    0x91, 0x02, //              Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xc0, //                End Collection
    // 1 bit padding
    0x75, 0x01, //          Report Size (1)
    0x95, 0x01, //          Report Count (1)
    0x91, 0x03, //          Output (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // RGB LED
    // Red
    0x05, 0x0a, //          Usage Page (Vendor Defined 0x0A)
    0x09, 0x07, //          Usage (0x07)
    0xa1, 0x02, //          Collection (Logical)
    0x05, 0x08, //              Usage Page (LEDs)
    0x09, 0x4b, //              Usage (Generic Indicator 1)
    0x79, 0x0b, //              String Index (11)
    0x75, 0x08, //              Report Size (8)
    0x95, 0x01, //              Report Count (1)
    0x91, 0x02, //              Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xc0, //                End Collection
    // Blue
    0x05, 0x0a, //          Usage Page (Vendor Defined 0x0A)
    0x09, 0x07, //          Usage (0x07)
    0xa1, 0x02, //          Collection (Logical)
    0x05, 0x08, //              Usage Page (LEDs)
    0x09, 0x4b, //              Usage (Generic Indicator 1)
    0x79, 0x0c, //              String Index (12)
    0x75, 0x08, //              Report Size (8)
    0x95, 0x01, //              Report Count (1)
    0x91, 0x02, //              Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xc0, //                End Collection
    // Green
    0x05, 0x0a, //          Usage Page (Vendor Defined 0x0A)
    0x09, 0x07, //          Usage (0x07)
    0xa1, 0x02, //          Collection (Logical)
    0x05, 0x08, //              Usage Page (LEDs)
    0x09, 0x4b, //              Usage (Generic Indicator 1)
    0x79, 0x0d, //              String Index (13)
    0x75, 0x08, //              Report Size (8)
    0x95, 0x01, //              Report Count (1)
    0x91, 0x02, //              Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xc0, //                End Collection

    // LED mode switch request
    0x85, 0x07, //          Report ID (7)
    0x05, 0x0a, //          Usage Page (Vendor Defined 0x0A)
    0x19, 0x00, //          Usage Minimum (0x00)
    0x29, 0x04, //          Usage Maximum (0x04)
    0x15, 0x00, //          Logical Minimum (0)
    0x25, 0x04, //          Logical Maximum (4)
    0x75, 0x04, //          Report Size (4)
    0x95, 0x01, //          Report Count (1)
    0xb1, 0x03, //          Feature (Const,Var,Abs)
    // 4 bits padding
    0x75, 0x04, //          Report Size (4)
    0x95, 0x01, //          Report Count (1)
    0xb1, 0x03, //          Feature (Const,Var,Abs)
    0xC0, //          End Collection
];

/// HID Input report for EAC mode
#[derive(Default, PartialEq, Eq)]
pub struct EacInputReport {
    /// Button states from button 1 to button 9
    pub buttons: [u8; 2],
    /// Analog axis
    pub axis: [i8; 2],
}

impl Serialize for EacInputReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_tuple(3)?;
        s.serialize_element(&4_u8)?; // Report ID
        s.serialize_element(&self.buttons)?;
        s.serialize_element(&self.axis)?;
        s.end()
    }
}

impl AsInputReport for EacInputReport {}

/// EAC LED control output report
#[derive(Default, PartialEq, Eq, FromBytes)]
pub struct EacOutputLedReport {
    pub led: u8,
    pub backlight: [u8; 3],
}

/// EAC LED mode control output report
#[derive(Default, PartialEq, Eq, FromBytes)]
pub struct EacOutputLedControlReport {
    pub mode: u8,
}
