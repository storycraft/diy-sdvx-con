use usbd_hid::descriptor::{SerializedDescriptor, generator_prelude::*};

/// HID report and descriptor for a gamepad with buttons and D-pad.
#[repr(C, packed)]
pub struct GamepadInputReport {
    /// Button states from button 1 to button 8
    pub buttons_0: u8,

    /// Button states from button 9 to button 16
    pub buttons_1: u8,

    /// D-pad state (0-8)
    /// 0: centered, 1: up, 2: up-left, 3: left, 4: down-left, 5: down, 6: down-right, 7: right, 8: up-right
    pub dpad: u8,
}

impl SerializedDescriptor for GamepadInputReport {
    #[rustfmt::skip]
    fn desc() -> &'static [u8] {
        &[
            0x05, 0x01, //      Usage Page (Generic Desktop Ctrls)
            0x09, 0x05, //      Usage (Game Pad)
            0xA1, 0x01, //      Collection (Application)
            0x05, 0x09, //          Usage Page (Button)
            0x19, 0x01, //          Usage Minimum (0x01)
            0x29, 0x10, //          Usage Maximum (0x10)
            0x15, 0x00, //          Logical Minimum (0)
            0x25, 0x01, //          Logical Maximum (1)
            0x75, 0x01, //          Report Size (1)
            0x95, 0x10, //          Report Count (16)
            0x81, 0x02, //          Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x05, 0x01, //          Usage Page (Generic Desktop Ctrls)
            0x09, 0x39, //          Usage (Hat switch)
            0x15, 0x01, //          Logical Minimum (1)
            0x25, 0x08, //          Logical Maximum (8)
            0x35, 0x00, //          Physical Minimum (0)
            0x46, 0x3B, 0x01, //    Physical Maximum (315)
            0x66, 0x14, 0x00, //    Unit (System: English Rotation, Length: Centimeter)
            0x75, 0x04, //          Report Size (4)
            0x95, 0x01, //          Report Count (1)
            0x81, 0x02, //          Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x75, 0x04, //          Report Size (4)
            0x95, 0x01, //          Report Count (1)
            0x15, 0x00, //          Logical Minimum (0)
            0x25, 0x00, //          Logical Maximum (0)
            0x35, 0x00, //          Physical Minimum (0)
            0x45, 0x00, //          Physical Maximum (0)
            0x65, 0x00, //          Unit (None)
            0x81, 0x03, //          Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0xC0, //          End Collection
        ]
    }
}

impl Serialize for GamepadInputReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_tuple(3)?;
        s.serialize_element(&self.buttons_0)?;
        s.serialize_element(&self.buttons_1)?;
        s.serialize_element(&self.dpad)?;
        s.end()
    }
}

impl AsInputReport for GamepadInputReport {}

/// Raw HID report compatible with QMK Raw HID.
#[repr(C, packed)]
pub struct QmkRawHidReport {
    pub data: [u8; 32],
}

impl SerializedDescriptor for QmkRawHidReport {
    #[rustfmt::skip]
    /// HID Descriptor copied from
    /// https://github.com/qmk/qmk_firmware/blob/acbeec29dab5331fe914f35a53d6b43325881e4d/tmk_core/protocol/vusb/vusb.c#L752
    fn desc() -> &'static [u8] {
        &[
            0x06, 0x60, 0xFF, // Usage Page (0xFF60 Vendor Defined)
            0x09, 0x61,       // Usage (0x61 Vendor Defined)
            0xA1, 0x01,       // Collection (Application)
            // Data to host
            0x09, 0x62,            //   Usage (Vendor Defined)
            0x15, 0x00,            //   Logical Minimum (0)
            0x26, 0xFF, 0x00,      //   Logical Maximum (255)
            0x95, 32,              //   Report Count
            0x75, 0x08,            //   Report Size (8)
            0x81, 0x02,            //   Input (Data, Variable, Absolute)
            // Data from host
            0x09, 0x63,            //   Usage (Vendor Defined)
            0x15, 0x00,            //   Logical Minimum (0)
            0x26, 0xFF, 0x00,      //   Logical Maximum (255)
            0x95, 32,              //   Report Count
            0x75, 0x08,            //   Report Size (8)
            0x91, 0x02,            //   Output (Data, Variable, Absolute)
            0xC0                   // End Collection
        ]
    }
}

impl Serialize for QmkRawHidReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_tuple(32)?;
        s.serialize_element(&self.data)?;
        s.end()
    }
}

impl AsInputReport for QmkRawHidReport {}
