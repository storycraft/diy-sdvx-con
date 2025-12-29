use zerocopy::{Immutable, IntoBytes, TryFromBytes};


#[derive(Clone, PartialEq, Eq, TryFromBytes, IntoBytes, Immutable)]
#[repr(C)]
pub struct Keymap {
    
}

impl Keymap {
    pub const DEFAULT: Self = Self {
        
    };
}

impl Default for Keymap {
    fn default() -> Self {
        Self::DEFAULT
    }
}