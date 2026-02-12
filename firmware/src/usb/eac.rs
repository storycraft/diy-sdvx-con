use usbd_hid::descriptor::{SerializedDescriptor, generator_prelude::*};

/// HID Report for EAC mode
#[derive(Default, PartialEq, Eq)]
pub struct EacInputReport {
    /// Button states from button 1 to button 9
    pub buttons: u16,
    /// Analog axis
    pub axis: [u8; 2],

}

impl SerializedDescriptor for EacInputReport {
    #[rustfmt::skip]
    fn desc() -> &'static [u8] {
        &[
            0x05, 0x01, //      Usage Page (Generic Desktop Ctrls)
            0x09, 0x04, //      Usage (Joystick)
            0xA1, 0x01, //      Collection (Application)
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
            0x95, 0x01, //          Report Count (1)
            0x75, 0x04, //          Report Size (4)
            0xb1, 0x03, //          Feature (Const,Var,Abs)
            0xC0, //          End Collection
        ]
    }
}

impl Serialize for EacInputReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_tuple(2)?;
        s.serialize_element(&self.buttons)?;
        s.serialize_element(&self.axis)?;
        s.end()
    }
}

impl AsInputReport for EacInputReport {}
