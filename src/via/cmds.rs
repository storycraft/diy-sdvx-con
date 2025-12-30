use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, big_endian};

#[derive(KnownLayout, Immutable, FromBytes, IntoBytes)]
#[repr(C)]
pub struct GetProtocolVersion {
    pub version: big_endian::U16,
}

impl GetProtocolVersion {
    /// Via protocol version from
    /// https://github.com/qmk/qmk_firmware/blob/acbeec29dab5331fe914f35a53d6b43325881e4d/quantum/via.h#L42
    pub const CURRENT_VERSION: u16 = 0x000C;
}

#[derive(KnownLayout, Immutable, FromBytes, IntoBytes)]
#[repr(C)]
pub struct DynamicKeymapKeycode {
    pub layer: u8,
    pub row: u8,
    pub col: u8,
    pub key: big_endian::U16,
}

#[derive(KnownLayout, Immutable, FromBytes, IntoBytes)]
#[repr(C)]
pub struct DynamicKeymapBuffer {
    pub offset: big_endian::U16,
    pub size: u8,
}

#[derive(KnownLayout, Immutable, FromBytes, IntoBytes)]
#[repr(C)]
pub struct DynamicKeymapEncoder {
    pub layer: u8,
    pub encoder_id: u8,
    pub clockwise: u8,
    pub key: big_endian::U16,
}
